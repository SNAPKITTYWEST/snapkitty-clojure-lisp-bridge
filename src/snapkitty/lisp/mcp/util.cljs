(ns snapkitty.lisp.mcp.util
  "Utilities — logging, IDs, response formatting"
  (:require [cljs.reader :as reader]
            ["crypto" :as crypto]))

(defn log [& msg]
  "Log to stderr (safe for stdout-based MCP transport)"
  (.error js/console (str/join " " msg)))

(defn uuid []
  "Generate a random UUID v4"
  (str (js/Math.random)))

(defn sha256 [data]
  "SHA-256 hash of data (string or buffer)"
  (-> (crypto/createHash "sha256")
      (.update (if (string? data) data (js/Buffer.from data)))
      (.digest "hex")))

(defn tool-result [content {:keys [is-error] :or {is-error false}}]
  "Format MCP tool result"
  {:type "text"
   :text content
   :is-error is-error})

(defn tool-error [message]
  "Format MCP tool error response"
  (tool-result message {:is-error true}))
