(ns doomsun.solarium.dashboard.static
  (:require ["node:fs" :as fs]
            ["node:path" :as path]))

(def app-js
  "(function() {
  function connectSSE(url, targetId) {
    var es = new EventSource(url);
    es.addEventListener('morph', function(e) {
      var target = document.getElementById(targetId);
      if (target && window.Idiomorph) {
        Idiomorph.morph(target, e.data, {morphStyle: 'innerHTML'});
      }
    });
    es.onerror = function() {
      es.close();
      setTimeout(function() { connectSSE(url, targetId); }, 3000);
    };
  }

  document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('[data-sse-url]').forEach(function(el) {
      connectSSE(el.dataset.sseUrl, el.id);
    });
  });
})();")

(defn- find-file [& candidates]
  (first (filter #(fs/existsSync %) candidates)))

(def app-css
  (let [css-path (find-file (path/resolve js/__dirname "dashboard.css")
                            (path/resolve js/__dirname "dashboard" "dashboard.css"))]
    (if css-path (.toString (fs/readFileSync css-path "utf-8")) "")))

(def logo-svg
  (let [logo-path (find-file (path/resolve js/__dirname ".." "logo.svg")
                             (path/resolve js/__dirname ".." ".." "logo.svg"))]
    (if logo-path (.toString (fs/readFileSync logo-path "utf-8")) "")))
