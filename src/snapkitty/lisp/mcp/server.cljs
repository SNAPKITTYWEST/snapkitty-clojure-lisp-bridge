(ns snapkitty.lisp.mcp.server
  "MCP server entry point"
  (:require ["@modelcontextprotocol/sdk/server/mcp.js" :refer [McpServer]]
            ["@modelcontextprotocol/sdk/server/stdio.js" :refer [StdioServerTransport]]
            [promesa.core :as p]
            [snapkitty.lisp.mcp.config :as config]
            [snapkitty.lisp.mcp.tools :as tools]
            [snapkitty.lisp.mcp.util :as util]
            [snapkitty.lisp.knowledge.qdrant :as qdrant]))

(defn start! []
  "Start MCP server on stdio transport"
  (let [cfg (config/load-config)
        mcp-server (McpServer. (clj->js {:name "snapkitty-lisp"
                                         :version "1.0.0"}))]
    (util/log "[snapkitty-lisp] Loading configuration...")
    (util/log "[snapkitty-lisp] Qdrant URL:" (:qdrant-url cfg))
    (util/log "[snapkitty-lisp] Collection:" (:collection-name cfg))

    (p/let [_ (qdrant/ensure-collection! cfg)]
      (util/log "[snapkitty-lisp] Collection ready")
      (tools/register-tools! mcp-server cfg)
      (util/log "[snapkitty-lisp] Tools registered")

      (let [transport (StdioServerTransport.)]
        (p/let [_ (.connect mcp-server transport)]
          (util/log "[snapkitty-lisp] MCP server ready on stdio"))))))

;; Node.js entry point
(set! (.-default js/module.exports) start!)
