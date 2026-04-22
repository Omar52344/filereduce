con archivos grande sigue pasando esto


[Worker] Converting EDIFACT to JSONL...
worker.mjs:35 [Worker] Large file detected (162.00 MB), using async conversion
filereduce_wasm.js:210 FileReduceWasm initialized
worker.mjs:59 [Worker] Conversion complete, output size: 595602766 bytes
worker.mjs:65 [Worker] First 100 bytes (hex): 7b 22 69 6e 74 65 72 63 68 61 6e 67 65 5f 69 64 22 3a 22 52 45 43 45 49 56 45 52 22 2c 22 73 65 6e 64 65 72 22 3a 22 33 22 2c 22 72 65 63 65 69 76 65 72 22 3a 22 53 45 4e 44 45 52 22 2c 22 64 6f 63 5f 74 79 70 65 22 3a 22 4f 52 44 45 52 53 22 2c 22 64 6f 63 75 6d 65 6e 74 5f 6e 75 6d 62 65 72 22 3a
worker.mjs:68 [Worker] First 200 chars as text: "{"interchange_id":"RECEIVER","sender":"3","receiver":"SENDER","doc_type":"ORDERS","document_number":"ORDER434868","document_date":"20260421","requested_delivery_date":null,"currency":"USD","buyer":"BU"
FileUpload.tsx:170 [processWithWorker] jsonlData type: object, isUint8Array: true, jsonlBytes size: 595602766
FileUpload.tsx:176 [processWithWorker] First 100 bytes (hex): 7b 22 69 6e 74 65 72 63 68 61 6e 67 65 5f 69 64 22 3a 22 52 45 43 45 49 56 45 52 22 2c 22 73 65 6e 64 65 72 22 3a 22 33 22 2c 22 72 65 63 65 69 76 65 72 22 3a 22 53 45 4e 44 45 52 22 2c 22 64 6f 63 5f 74 79 70 65 22 3a 22 4f 52 44 45 52 53 22 2c 22 64 6f 63 75 6d 65 6e 74 5f 6e 75 6d 62 65 72 22 3a
FileUpload.tsx:180 [processWithWorker] First 200 chars as text: "{"interchange_id":"RECEIVER","sender":"3","receiver":"SENDER","doc_type":"ORDERS","document_number":"ORDER434868","document_date":"20260421","requested_delivery_date":null,"currency":"USD","buyer":"BU"
FileUpload.tsx:85 [parseJsonlPartial] Starting parse of 595602766 bytes
FileUpload.tsx:123 [parseJsonlPartial] Result: totalLines=1, linesProcessed=0, linesWithContent=0, documentsFound=0, jsonParseErrors=0
FileUpload.tsx:192 [processWithWorker] Parse results: documents=0, totalLines=1, linesProcessed=0
worker.mjs:90 [Worker] Compressing JSONL to .fra...
worker.mjs:99 [Worker] JSONL compression error: RuntimeError: unreachable
    at filereduce_wasm.wasm.std::alloc::rust_oom::{{closure}}::hd75745f6cab4b1f8 (filereduce_wasm_bg.wasm?v=1776820441205:0xc6fe7)
    at filereduce_wasm.wasm.std::sys::backtrace::__rust_end_short_backtrace::h81a6ec13af23b259 (filereduce_wasm_bg.wasm?v=1776820441205:0xc7e85)
    at filereduce_wasm.wasm.std::alloc::rust_oom::hbbbc0258d349aa94 (filereduce_wasm_bg.wasm?v=1776820441205:0xc67e9)
    at filereduce_wasm.wasm.__rustc[16f1505adc47261a]::__rust_alloc_error_handler (filereduce_wasm_bg.wasm?v=1776820441205:0xc7d17)
    at filereduce_wasm.wasm.alloc::alloc::handle_alloc_error::hec8d3aa2a30efaa7 (filereduce_wasm_bg.wasm?v=1776820441205:0xc7d72)
    at filereduce_wasm.wasm.alloc::raw_vec::handle_error::h9ace31a903e6893e (filereduce_wasm_bg.wasm?v=1776820441205:0xc7405)
    at filereduce_wasm.wasm.<alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter::h89adc4b0f9115aac (filereduce_wasm_bg.wasm?v=1776820441205:0x8d000)
    at filereduce_wasm.wasm.filereducelib::FileReduceCompressor::json_to_binary::h4ec1bea20332b49c (filereduce_wasm_bg.wasm?v=1776820441205:0x9d369)
    at filereduce_wasm.wasm.<core::iter::adapters::map::Map<I,F> as core::iter::traits::iterator::Iterator>::fold::h5eb62ce83f0a16bf (filereduce_wasm_bg.wasm?v=1776820441205:0xb7146)
    at filereduce_wasm.wasm.alloc::vec::Vec<T,A>::extend_trusted::h1ec7ef6259c9aeb7 (filereduce_wasm_bg.wasm?v=1776820441205:0xbcfff)
compressJsonl @ worker.mjs:99
await in compressJsonl
(anonymous) @ worker.mjs:141
Worker.postMessage
(anonymous) @ wasmWorker.ts:77
sendRequest @ wasmWorker.ts:75
await in sendRequest
compressJsonl @ wasmWorker.ts:117
(anonymous) @ FileUpload.tsx:287
await in (anonymous)
executeDispatch @ react-dom-client.development.js:20610
runWithFiberInDEV @ react-dom-client.development.js:986
processDispatchQueue @ react-dom-client.development.js:20660
(anonymous) @ react-dom-client.development.js:21234
batchedUpdates$1 @ react-dom-client.development.js:3377
dispatchEventForPluginEventSystem @ react-dom-client.development.js:20814
dispatchEvent @ react-dom-client.development.js:25817
dispatchDiscreteEvent @ react-dom-client.development.js:25785
<button>
(anonymous) @ react-jsx-dev-runtime.development.js:342
FileUpload @ FileUpload.tsx:451
react_stack_bottom_frame @ react-dom-client.development.js:28241
renderWithHooksAgain @ react-dom-client.development.js:8025
renderWithHooks @ react-dom-client.development.js:7937
updateFunctionComponent @ react-dom-client.development.js:10442
beginWork @ react-dom-client.development.js:12112
runWithFiberInDEV @ react-dom-client.development.js:986
performUnitOfWork @ react-dom-client.development.js:18988
workLoopSync @ react-dom-client.development.js:18816
renderRootSync @ react-dom-client.development.js:18797
performWorkOnRoot @ react-dom-client.development.js:17823
performSyncWorkOnRoot @ react-dom-client.development.js:20486
flushSyncWorkAcrossRoots_impl @ react-dom-client.development.js:20328
processRootScheduleInMicrotask @ react-dom-client.development.js:20367
(anonymous) @ react-dom-client.development.js:20505
installHook.js:1 Translation key not found: dashboard.undefined
overrideMethod @ installHook.js:1
(anonymous) @ forward-logs-shared.ts:95
(anonymous) @ LanguageContext.tsx:321
(anonymous) @ Dashboard.tsx:26
Dashboard @ Dashboard.tsx:87
react_stack_bottom_frame @ react-dom-client.development.js:28241
renderWithHooks @ react-dom-client.development.js:7925
updateFunctionComponent @ react-dom-client.development.js:10442
beginWork @ react-dom-client.development.js:12112
runWithFiberInDEV @ react-dom-client.development.js:986
performUnitOfWork @ react-dom-client.development.js:18988
workLoopSync @ react-dom-client.development.js:18816
renderRootSync @ react-dom-client.development.js:18797
performWorkOnRoot @ react-dom-client.development.js:17823
performWorkOnRootViaSchedulerTask @ react-dom-client.development.js:20471
performWorkUntilDeadline @ scheduler.development.js:45
<Dashboard>
(anonymous) @ react-jsx-dev-runtime.development.js:342
FileUpload @ FileUpload.tsx:478
react_stack_bottom_frame @ react-dom-client.development.js:28241
renderWithHooksAgain @ react-dom-client.development.js:8025
renderWithHooks @ react-dom-client.development.js:7937
updateFunctionComponent @ react-dom-client.development.js:10442
beginWork @ react-dom-client.development.js:12112
runWithFiberInDEV @ react-dom-client.development.js:986
performUnitOfWork @ react-dom-client.development.js:18988
workLoopSync @ react-dom-client.development.js:18816
renderRootSync @ react-dom-client.development.js:18797
performWorkOnRoot @ react-dom-client.development.js:17823
performWorkOnRootViaSchedulerTask @ react-dom-client.development.js:20471
performWorkUntilDeadline @ scheduler.development.js:45
installHook.js:1 Translation key not found: dashboard.undefined

Cuando se pasa de edi a json y se pide que tambien  a .fra directametne


y sigue pasando lo mismo

edifact-D13B-10MB.fra
Tipo: fra • Tamaño: 3900.24 KB • Operación: Descomprimir a JSONL

Eliminar
Descomprimir a JSONL solamente funciona en el link de compression la descompresion pero no sirve compresion de ningun tamaño