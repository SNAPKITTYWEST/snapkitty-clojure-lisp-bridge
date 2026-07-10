(ns doomsun.solarium.server
  (:require ["@modelcontextprotocol/sdk/server/mcp.js" :refer [McpServer]]
            ["zod" :refer [z]]
            [doomsun.solarium.tools.documents :as docs]
            [doomsun.solarium.tools.search :as search]
            [doomsun.solarium.tools.tags :as tags]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn- wrap-handler
  "Wrap a tool handler with error handling."
  [config handler-fn]
  (fn [params _extra]
    (let [args (js->clj params :keywordize-keys true)]
      (p/catch
       (p/let [result (handler-fn config args)]
         (util/tool-result result))
       (fn [err]
         (util/log "Tool error:" (.-message err))
         (util/tool-error (str "Error: " (.-message err))))))))

(defn register-tools!
  "Register all MCP tools on the server."
  [^js server config]

  ;; store_document
  (.tool server "store_document"
         "Store a document in the knowledge base. Chunks and embeds the content for semantic search."
         (clj->js {:title      (.describe (.string z) "Document title")
                   :content    (.describe (.string z) "Full document content")
                   :tags       (.optional (.array z (.string z)))
                   :source_url (.optional (.string z))})
         (wrap-handler config
                       (fn [cfg args] (docs/store-document! cfg args))))

  ;; search
  (.tool server "search"
         "Semantic search across all documents. Returns ranked results with snippets."
         (clj->js {:query (.describe (.string z) "Search query")
                   :limit (.optional (.number z))})
         (wrap-handler config
                       (fn [cfg args] (search/search-documents cfg args))))

  ;; list_documents
  (.tool server "list_documents"
         "List all documents in the knowledge base with metadata."
         (clj->js {:limit  (.optional (.number z))
                   :offset (.optional (.string z))})
         (wrap-handler config
                       (fn [cfg args] (docs/list-documents cfg args))))

  ;; read_document
  (.tool server "read_document"
         "Read the full content of a document by its ID."
         (clj->js {:id (.describe (.string z) "Document ID")})
         (wrap-handler config
                       (fn [cfg {:keys [id]}]
                         (p/let [doc (docs/read-document cfg id)]
                           (or doc {:error "Document not found"})))))

  ;; update_document
  (.tool server "update_document"
         "Update a document's content or metadata. Re-embeds if content changes."
         (clj->js {:id         (.describe (.string z) "Document ID")
                   :title      (.optional (.string z))
                   :content    (.optional (.string z))
                   :tags       (.optional (.array z (.string z)))
                   :source_url (.optional (.string z))})
         (wrap-handler config
                       (fn [cfg args] (docs/update-document! cfg args))))

  ;; delete_document
  (.tool server "delete_document"
         "Delete a document and all its chunks from the knowledge base."
         (clj->js {:id (.describe (.string z) "Document ID")})
         (wrap-handler config
                       (fn [cfg {:keys [id]}] (docs/delete-document! cfg id))))

  ;; list_tags
  (.tool server "list_tags"
         "List all tags used across documents with counts."
         (clj->js {})
         (wrap-handler config
                       (fn [cfg _args] (tags/list-tags cfg))))

  ;; tag_document
  (.tool server "tag_document"
         "Add tags to a document."
         (clj->js {:id   (.describe (.string z) "Document ID")
                   :tags (.describe (.array z (.string z)) "Tags to add")})
         (wrap-handler config
                       (fn [cfg args] (tags/tag-document! cfg args))))

  ;; untag_document
  (.tool server "untag_document"
         "Remove tags from a document."
         (clj->js {:id   (.describe (.string z) "Document ID")
                   :tags (.describe (.array z (.string z)) "Tags to remove")})
         (wrap-handler config
                       (fn [cfg args] (tags/untag-document! cfg args)))))
