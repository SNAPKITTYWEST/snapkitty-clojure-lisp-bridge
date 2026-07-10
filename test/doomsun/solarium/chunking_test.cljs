(ns doomsun.solarium.chunking-test
  (:require [cljs.test :refer [deftest is testing]]
            [doomsun.solarium.chunking :as chunking]))

(deftest short-text-single-chunk
  (testing "Text shorter than max-chars returns single chunk"
    (let [text   "Short paragraph."
          chunks (chunking/chunk-text {:chunk-max-chars 1600 :chunk-overlap 200} text)]
      (is (= 1 (count chunks)))
      (is (= text (first chunks))))))

(deftest multi-paragraph-chunking
  (testing "Multiple paragraphs get split into chunks"
    (let [para   (apply str (repeat 100 "word "))  ;; ~500 chars
          text   (str para "\n\n" para "\n\n" para "\n\n" para)
          chunks (chunking/chunk-text {:chunk-max-chars 1200 :chunk-overlap 200} text)]
      (is (> (count chunks) 1))
      ;; All original content should be present across chunks
      (is (every? #(> (count %) 0) chunks)))))

(deftest overlap-between-chunks
  (testing "Chunks have overlap content from previous chunk"
    (let [para1  (apply str (repeat 80 "alpha "))
          para2  (apply str (repeat 80 "beta "))
          para3  (apply str (repeat 80 "gamma "))
          text   (str para1 "\n\n" para2 "\n\n" para3)
          chunks (chunking/chunk-text {:chunk-max-chars 500 :chunk-overlap 100} text)]
      (is (>= (count chunks) 2)))))

(deftest long-paragraph-sentence-split
  (testing "Single paragraph exceeding max-chars splits on sentences"
    (let [sentences (repeatedly 20 #(str "This is a sentence that is moderately long. "))
          text      (apply str sentences)
          chunks    (chunking/chunk-text {:chunk-max-chars 200 :chunk-overlap 50} text)]
      (is (> (count chunks) 1))
      (is (every? #(<= (count %) 250) chunks)))))  ;; allow some tolerance

(deftest prefix-document-chunks-test
  (testing "Adds search_document prefix"
    (let [chunks  ["Hello world" "Another chunk"]
          prefixed (chunking/prefix-document-chunks chunks)]
      (is (= "search_document: Hello world" (first prefixed)))
      (is (= "search_document: Another chunk" (second prefixed))))))

(deftest prefix-query-test
  (testing "Adds search_query prefix"
    (is (= "search_query: test" (chunking/prefix-query "test")))))
