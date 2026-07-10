(ns doomsun.solarium.chunking
  (:require [clojure.string :as str]))

(defn- split-sentences
  "Split text on sentence boundaries (period/question/exclamation followed by space)."
  [text]
  (let [parts (.split text #"(?<=[.!?])\s+")]
    (vec (remove str/blank? parts))))

(defn- split-paragraphs
  "Split on double newlines."
  [text]
  (vec (remove str/blank? (.split text #"\n\n+"))))

(defn chunk-text
  "Split text into chunks of approximately max-chars with overlap.
   Paragraph-aware: tries to keep paragraphs intact.
   Returns a vector of chunk strings."
  [{:keys [chunk-max-chars chunk-overlap]
    :or   {chunk-max-chars 1600
           chunk-overlap   200}}
   text]
  (let [paragraphs (split-paragraphs text)]
    (if (<= (count text) chunk-max-chars)
      [text]
      (loop [paras     paragraphs
             current   ""
             carry     ""
             chunks    []]
        (if (empty? paras)
          (if (str/blank? current)
            chunks
            (conj chunks (str/trim current)))
          (let [para     (first paras)
                combined (if (str/blank? current)
                           (str carry (when-not (str/blank? carry) "\n\n") para)
                           (str current "\n\n" para))]
            (cond
              ;; Paragraph fits in current chunk
              (<= (count combined) chunk-max-chars)
              (recur (rest paras) combined carry chunks)

              ;; Current chunk is non-empty, finalize it and start new with overlap
              (not (str/blank? current))
              (let [trimmed   (str/trim current)
                    ;; Take last ~overlap chars as carry
                    carry-start (max 0 (- (count trimmed) chunk-overlap))
                    new-carry   (subs trimmed carry-start)]
                (recur paras "" new-carry (conj chunks trimmed)))

              ;; Single paragraph exceeds max — split on sentences
              :else
              (let [sentences  (split-sentences para)
                    sub-chunks (loop [sents    sentences
                                      cur      (str carry)
                                      sub-acc  []]
                                 (if (empty? sents)
                                   (if (str/blank? cur)
                                     sub-acc
                                     (conj sub-acc (str/trim cur)))
                                   (let [sent      (first sents)
                                         combined  (str cur (when-not (str/blank? cur) " ") sent)]
                                     (if (<= (count combined) chunk-max-chars)
                                       (recur (rest sents) combined sub-acc)
                                       (if (str/blank? cur)
                                         ;; Single sentence exceeds max, force include
                                         (recur (rest sents) "" (conj sub-acc (str/trim sent)))
                                         (recur sents "" (conj sub-acc (str/trim cur))))))))]
                (let [last-chunk  (peek sub-chunks)
                      carry-start (max 0 (- (count last-chunk) chunk-overlap))
                      new-carry   (subs last-chunk carry-start)]
                  (recur (rest paras) "" new-carry (into chunks sub-chunks)))))))))))

(defn prefix-document-chunks
  "Add 'search_document: ' prefix to chunks for nomic model."
  [chunks]
  (mapv #(str "search_document: " %) chunks))

(defn prefix-query
  "Add 'search_query: ' prefix for nomic model."
  [query]
  (str "search_query: " query))
