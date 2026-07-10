(ns doomsun.solarium.dashboard.http
  "HTTP response helpers for the dashboard server.")

(defn respond-html [^js res status html-str]
  (.writeHead res status #js {"Content-Type" "text/html; charset=utf-8"})
  (.end res html-str))

(defn respond-json [^js res status data]
  (.writeHead res status #js {"Content-Type" "application/json"})
  (.end res (js/JSON.stringify (clj->js data))))

(defn respond-redirect [^js res location]
  (.writeHead res 302 #js {"Location" location})
  (.end res))

(defn respond-sse [^js res]
  (.writeHead res 200 #js {"Content-Type"  "text/event-stream"
                           "Cache-Control" "no-cache"
                           "Connection"    "keep-alive"})
  (.write res ":\n\n")
  res)

(defn respond-static [^js res content-type body]
  (.writeHead res 200 #js {"Content-Type" content-type
                           "Cache-Control" "public, max-age=3600"})
  (.end res body))
