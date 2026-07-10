(ns doomsun.solarium.main
  (:require ["@modelcontextprotocol/sdk/server/mcp.js" :refer [McpServer]]
            ["@modelcontextprotocol/sdk/server/stdio.js" :refer [StdioServerTransport]]
            [doomsun.solarium.config :as config]
            [doomsun.solarium.server :as server]
            [doomsun.solarium.qdrant.collections :as collections]
            [doomsun.solarium.util :as util]
            [promesa.core :as p]))

(defn main! []
  (let [cfg       (config/load-config)
        mcp-server (McpServer. (clj->js {:name    "solarium"
                                         :version "0.1.0"}))]
    (server/register-tools! mcp-server cfg)

    (p/let [_ (collections/ensure-collection! cfg)]
      (let [transport (StdioServerTransport.)]
        (p/let [_ (.connect mcp-server transport)]
          (util/log "Solarium MCP server running on stdio."))))))
