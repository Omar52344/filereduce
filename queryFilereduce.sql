-- Verificar si las tablas existen antes de crearlas
IF OBJECT_ID('tbl_EDI_Orders_Lines', 'U') IS NOT NULL DROP TABLE tbl_EDI_Orders_Lines;
IF OBJECT_ID('tbl_EDI_Orders_Header', 'U') IS NOT NULL DROP TABLE tbl_EDI_Orders_Header;
GO

-- 1. Tabla de Cabeceras
CREATE TABLE dbo.tbl_EDI_Orders_Header (
    Order_Id INT IDENTITY(1,1) PRIMARY KEY,
    Interchange_Id NVARCHAR(50) NOT NULL,
    Sender_Id NVARCHAR(50) NOT NULL,
    Receiver_Id NVARCHAR(50) NOT NULL,
    Doc_Type NVARCHAR(20),             -- Ejemplo: ORDERS
    Document_Number NVARCHAR(50) NOT NULL,
    Document_Date DATE NOT NULL,
    Requested_Delivery_Date DATE NULL,
    Currency NVARCHAR(5) DEFAULT 'UNKNOWN',
    Buyer_Name NVARCHAR(150),
    Seller_Name NVARCHAR(150),
    Line_Count_Check INT,              -- Validado contra el CNT+2
    ProcessedAt DATETIME DEFAULT GETDATE(),
    
    -- Restricción para evitar duplicidad de mensajes
    CONSTRAINT UQ_EDI_Order UNIQUE (Sender_Id, Document_Number, Interchange_Id)
);

-- 2. Tabla de Detalles
CREATE TABLE dbo.tbl_EDI_Orders_Lines (
    Line_Id INT IDENTITY(1,1) PRIMARY KEY,
    Order_Id INT NOT NULL,              -- Relación con Header
    Line_No INT NOT NULL,               -- Posición original (LIN+1, LIN+2...)
    SKU NVARCHAR(100) NOT NULL,
    Qty DECIMAL(18,4) NOT NULL,         -- Soporta decimales (Kilos, Litros)
    UOM NVARCHAR(10),                   -- Unidad de medida (EA, KGM, etc.)
    Amount DECIMAL(18,2) NOT NULL,      -- Monto de la línea
    
    CONSTRAINT FK_Order_Lines_Header FOREIGN KEY (Order_Id) 
        REFERENCES tbl_EDI_Orders_Header(Order_Id) ON DELETE CASCADE
);
GO


CREATE OR ALTER PROCEDURE dbo.sp_EDI_Ingresar_Batch_Orders
    @JsonBatch NVARCHAR(MAX) -- Recibe el [{}, {}, ...] de hasta 1000 registros
AS
BEGIN
    SET NOCOUNT ON;
    
    
    
    -- 1. Tabla temporal para depositar los objetos del array
    -- Usamos el [key] de OPENJSON como índice único del lote
    DECLARE @OrdersToProcess TABLE (
        BatchIndex INT PRIMARY KEY,
        SingleOrderJson NVARCHAR(MAX)
    );

    -- Cargamos el array en la tabla temporal
    INSERT INTO @OrdersToProcess (BatchIndex, SingleOrderJson)
    SELECT [key], [value] 
    FROM OPENJSON(@JsonBatch);

    -- Variables para el control del bucle
    DECLARE @Index INT = 0;
    DECLARE @Total INT = (SELECT COUNT(*) FROM @OrdersToProcess);
    DECLARE @CurrentJson NVARCHAR(MAX);
    DECLARE @NewOrderId INT;

    -- 2. Iniciamos el procesamiento del lote
    WHILE @Index < @Total
    BEGIN
        SELECT @CurrentJson = SingleOrderJson 
        FROM @OrdersToProcess 
        WHERE BatchIndex = @Index;

        BEGIN TRY
            BEGIN TRANSACTION;

                -- Insertar Cabecera de la orden actual
                INSERT INTO dbo.tbl_EDI_Orders_Header (
                    Interchange_Id, Sender_Id, Receiver_Id, Doc_Type, 
                    Document_Number, Document_Date, Requested_Delivery_Date, 
                    Currency, Buyer_Name, Seller_Name, Line_Count_Check
                )
                SELECT 
                    interchange_id, sender, receiver, doc_type,
                    document_number, document_date, requested_delivery_date,
                    currency, buyer, seller, line_count_check
                FROM OPENJSON(@CurrentJson)
                WITH (
                    interchange_id NVARCHAR(50),
                    sender NVARCHAR(50),
                    receiver NVARCHAR(50),
                    doc_type NVARCHAR(20),
                    document_number NVARCHAR(50),
                    document_date DATE,
                    requested_delivery_date DATE,
                    currency NVARCHAR(5),
                    buyer NVARCHAR(150),
                    seller NVARCHAR(150),
                    line_count_check INT
                );

                -- Capturamos el ID generado para esta orden específica
                SET @NewOrderId = SCOPE_IDENTITY();

                -- Insertar los detalles de esta orden
                IF @NewOrderId IS NOT NULL
                BEGIN
                    INSERT INTO dbo.tbl_EDI_Orders_Lines (
                        Order_Id, Line_No, SKU, Qty, UOM, Amount
                    )
                    SELECT 
                        @NewOrderId, line_no, sku, qty, uom, amount
                    FROM OPENJSON(@CurrentJson, '$.lines')
                    WITH (
                        line_no INT,
                        sku NVARCHAR(100),
                        qty DECIMAL(18,4),
                        uom NVARCHAR(10),
                        amount DECIMAL(18,2)
                    );
                END

            COMMIT TRANSACTION;
        END TRY
        BEGIN CATCH
            -- Si una orden falla (ej. duplicado), hacemos rollback solo de esa orden
            IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION;

            -- Logueamos el error pero permitimos que el bucle siga con la siguiente orden
            IF ERROR_NUMBER() IN (2601, 2627)
                PRINT 'Orden index ' + CAST(@Index AS VARCHAR) + ' omitida por duplicado.';
            ELSE
                PRINT 'Error en index ' + CAST(@Index AS VARCHAR) + ': ' + ERROR_MESSAGE();
        END CATCH

        SET @Index = @Index + 1;
    END
END;
GO



CREATE OR ALTER PROCEDURE dbo.sp_EDI_Ingresar_Batch_Orders
    @JsonBatch NVARCHAR(MAX)
AS
BEGIN
    SET NOCOUNT ON;
    
    BEGIN TRY
        BEGIN TRANSACTION;

        INSERT INTO Log_Edifact (Registro)
        SELECT @JsonBatch

            -- 1. Inserción Masiva de Cabeceras
            -- Ignoramos los duplicados automáticamente gracias al CONSTRAINT UNIQUE y una validación
            INSERT INTO dbo.tbl_EDI_Orders_Header (
                Interchange_Id, Sender_Id, Receiver_Id, Doc_Type, 
                Document_Number, Document_Date, Requested_Delivery_Date, 
                Currency, Buyer_Name, Seller_Name, Line_Count_Check
            )
            SELECT 
                j.interchange_id, j.sender, j.receiver, j.doc_type,
                j.document_number, j.document_date, j.requested_delivery_date,
                j.currency, j.buyer, j.seller, j.line_count_check
            FROM OPENJSON(@JsonBatch)
            WITH (
                interchange_id NVARCHAR(50),
                sender NVARCHAR(50),
                receiver NVARCHAR(50),
                doc_type NVARCHAR(20),
                document_number NVARCHAR(50),
                document_date DATE,
                requested_delivery_date DATE,
                currency NVARCHAR(5),
                buyer NVARCHAR(150),
                seller NVARCHAR(150),
                line_count_check INT
            ) AS j
            WHERE NOT EXISTS (
                SELECT 1 FROM dbo.tbl_EDI_Orders_Header h
                WHERE h.Sender_Id = j.sender 
                  AND h.Document_Number = j.document_number 
                  AND h.Interchange_Id = j.interchange_id
            );

            -- 2. Inserción Masiva de Detalles (Lines)
            -- Unimos el JSON con la tabla de cabeceras usando los campos únicos
            INSERT INTO dbo.tbl_EDI_Orders_Lines (
                Order_Id, Line_No, SKU, Qty, UOM, Amount
            )
            SELECT 
                h.Order_Id, 
                l.line_no, 
                l.sku, 
                l.qty, 
                l.uom, 
                l.amount
            FROM OPENJSON(@JsonBatch)
            WITH (
                sender NVARCHAR(50),
                document_number NVARCHAR(50),
                interchange_id NVARCHAR(50),
                lines_data NVARCHAR(MAX) '$.lines' AS JSON -- Extraemos el array interno
            ) AS j
            JOIN dbo.tbl_EDI_Orders_Header h ON 
                h.Sender_Id = j.sender AND 
                h.Document_Number = j.document_number AND 
                h.Interchange_Id = j.interchange_id
            CROSS APPLY OPENJSON(j.lines_data)
            WITH (
                line_no INT,
                sku NVARCHAR(100),
                qty DECIMAL(18,4),
                uom NVARCHAR(10),
                amount DECIMAL(18,2)
            ) AS l
            WHERE NOT EXISTS (
                -- Evitamos duplicar líneas si el SP se corre dos veces para el mismo lote
                SELECT 1 FROM dbo.tbl_EDI_Orders_Lines dl
                WHERE dl.Order_Id = h.Order_Id AND dl.Line_No = l.line_no
            );

        COMMIT TRANSACTION;
    END TRY
    BEGIN CATCH
        IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION;
        
        DECLARE @ErrorMessage NVARCHAR(4000) = ERROR_MESSAGE();
        RAISERROR (@ErrorMessage, 16, 1);
    END CATCH
END;
GO


select * from tbl_EDI_Orders_Header
select *   from tbl_EDI_Orders_Lines
select * from Log_Edifact

--delete from tbl_EDI_Orders_Header
--delete from tbl_EDI_Orders_Lines