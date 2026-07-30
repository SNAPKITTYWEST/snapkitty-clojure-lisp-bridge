(ns snapkitty.lisp.mcp.tools
  "MCP tool definitions with Zod input validation"
  (:require [promesa.core :as p]
            [zod :as z]
            [snapkitty.lisp.mcp.util :as util]
            [snapkitty.lisp.knowledge.store :as store]))

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
                 :handler handle-delete-document}]]
    (.addTool mcp-server tool)))
