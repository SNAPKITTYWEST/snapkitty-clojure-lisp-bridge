(ns snapkitty.lisp.mcp.tools
  "MCP tool definitions with Zod input validation"
  (:require [promesa.core :as p]
            [zod :as z]
            [snapkitty.lisp.mcp.util :as util]
            [snapkitty.lisp.knowledge.store :as store]
            [snapkitty.lisp.native :as native]
            [snapkitty.lisp.emojiscript :as emoji]))

;; === INPUT SCHEMAS (Zod Validation) ===

(def store-document-schema
  (z/object
    {:id (z/string)
     :title (z/string)
     :content (z/string)
     :tags (z/array (z/string) {:optional true})}))

(def search-schema
  (z/object
    {:query (z/string)
     :limit (z/number {:optional true :default 10})
     :tags (z/array (z/string) {:optional true})}))

(def delete-document-schema
  (z/object
    {:id (z/string)}))

(def validate-mutation-schema
  (z/object
    {:mutation-id (z/number)
     :generation-before (z/number)
     :generation-after (z/number)
     :actor (z/number)
     :target (z/number)
     :operation (z/number {:optional true})
     :old-digest (z/string {:optional true})
     :new-digest (z/string {:optional true})}))

(def verify-blake3-schema
  (z/object
    {:payload (z/string)
     :expected-digest (z/string)}))

(def verify-ed25519-schema
  (z/object
    {:message (z/string)
     :signature (z/string)
     :public-key (z/string)}))

(def compile-emojiscript-schema
  (z/object
    {:source (z/string)}))

(def execute-emojiscript-schema
  (z/object
    {:source (z/string)
     :max-steps (z/number {:optional true :default 10000})}))

;; === TOOL HANDLERS ===

(defn handle-store-document [input]
  "Store document with embedding — INPUT VALIDATED"
  (p/let [validated (z/parse store-document-schema input)
          {:keys [id title content tags]} validated
          result (store/upsert! id {:title title
                                     :content content
                                     :tags (or tags [])})]
    (util/tool-result (str "Stored document " id))))

(defn handle-search [input]
  "Search knowledge base — INPUT VALIDATED"
  (p/let [validated (z/parse search-schema input)
          {:keys [query limit tags]} validated
          results (store/search query {:limit limit :tags tags})]
    (util/tool-result (str "Found " (count results) " documents"))))

(defn handle-delete-document [input]
  "Delete document — INPUT VALIDATED"
  (p/let [validated (z/parse delete-document-schema input)
          {:keys [id]} validated
          _ (store/delete! id)]
    (util/tool-result (str "Deleted document " id))))

(defn handle-validate-mutation [input]
  "Validate Lisp mutation with native NASM gate — 8-point validation"
  (p/let [validated (z/parse validate-mutation-schema input)
          result (native/validate-mutation! validated nil)]
    (util/tool-result (str "Mutation validation: "
                           (if (:passes-gate result)
                             "PASS"
                             (str "FAIL — " (:details result)))))))

(defn handle-verify-blake3 [input]
  "Verify Blake3 digest using native ASM — fast-path verification"
  (p/let [validated (z/parse verify-blake3-schema input)
          {:keys [payload expected-digest]} validated
          result (native/verify-blake3! payload expected-digest)]
    (util/tool-result (str "Blake3 verification: "
                           (if (:digest-valid result)
                             "MATCH"
                             (str "MISMATCH — " (:details result)))))))

(defn handle-verify-ed25519 [input]
  "Verify Ed25519 signature using native ASM — fast-path verification"
  (p/let [validated (z/parse verify-ed25519-schema input)
          {:keys [message signature public-key]} validated
          result (native/verify-ed25519! message signature public-key)]
    (util/tool-result (str "Ed25519 verification: "
                           (if (:signature-valid result)
                             "VALID"
                             (str "INVALID — " (:details result)))))))

(defn handle-compile-emojiscript [input]
  "Compile Ahmad's EmojiScript to bytecode — emoji tokens → SigilOp ops"
  (p/let [validated (z/parse compile-emojiscript-schema input)
          {:keys [source]} validated]
    (try
      (let [result (emoji/compile-emojiscript source)]
        (util/tool-result
          (str "✅ Compiled: " (:instructions-count result) " instructions\n"
               "Hash: " (subs (:hash result) 0 16) "...\n"
               "Bytecode: " (pr-str (:bytecode result)))))
      (catch js/Error e
        (util/tool-result (str "❌ Compilation failed: " (.-message e)))))))

(defn handle-execute-emojiscript [input]
  "Execute Ahmad's EmojiScript bytecode — stack-based interpreter"
  (p/let [validated (z/parse execute-emojiscript-schema input)
          {:keys [source max-steps]} validated
          compiled (try
                     (emoji/compile-emojiscript source)
                     (catch js/Error e
                       (throw (ex-info "Compile failed" {:error (.-message e)}))))
          result (emoji/execute-emojiscript (:bytecode compiled) :max-steps max-steps)]
    (util/tool-result
      (if (:error result)
        (str "❌ Runtime error: " (:error result))
        (str "✅ Result: " (:result result) "\n"
             "Stack: " (pr-str (:stack result)) "\n"
             "Steps: " (:steps result) " / " max-steps)))))

;; === TOOL REGISTRATION ===

(defn register-tools! [mcp-server cfg]
  "Register all MCP tools with the server"
  (doseq [tool [{:name "store_document"
                 :description "Store a document with semantic embedding"
                 :inputSchema store-document-schema
                 :handler handle-store-document}
                {:name "search"
                 :description "Semantic search across knowledge base"
                 :inputSchema search-schema
                 :handler handle-search}
                {:name "delete_document"
                 :description "Delete a document by ID"
                 :inputSchema delete-document-schema
                 :handler handle-delete-document}
                {:name "validate_mutation"
                 :description "Fast-path mutation validation gate (8-point check) via NASM"
                 :inputSchema validate-mutation-schema
                 :handler handle-validate-mutation}
                {:name "verify_blake3"
                 :description "Blake3 digest verification via native ASM"
                 :inputSchema verify-blake3-schema
                 :handler handle-verify-blake3}
                {:name "verify_ed25519"
                 :description "Ed25519 signature verification via native ASM"
                 :inputSchema verify-ed25519-schema
                 :handler handle-verify-ed25519}
                {:name "compile_emojiscript"
                 :description "Ahmad's EmojiScript → bytecode compiler (🔢6 🔢7 ✖️ ↩️)"
                 :inputSchema compile-emojiscript-schema
                 :handler handle-compile-emojiscript}
                {:name "execute_emojiscript"
                 :description "Ahmad's EmojiScript bytecode executor (stack-based VM)"
                 :inputSchema execute-emojiscript-schema
                 :handler handle-execute-emojiscript}]]
    (.addTool mcp-server tool)))
