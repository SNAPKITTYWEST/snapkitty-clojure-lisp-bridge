(ns doomsun.solarium.dashboard.html-test
  (:require [cljs.test :refer [deftest is testing]]
            [doomsun.solarium.dashboard.html :as html]))

(deftest renders-simple-element
  (is (= "<div>hello</div>" (html/render [:div "hello"]))))

(deftest renders-attributes
  (is (= "<div class=\"foo\">bar</div>"
         (html/render [:div {:class "foo"} "bar"]))))

(deftest renders-nested-elements
  (is (= "<ul><li>a</li><li>b</li></ul>"
         (html/render [:ul [:li "a"] [:li "b"]]))))

(deftest escapes-html-content
  (is (= "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>"
         (html/render [:p "<script>alert(1)</script>"]))))

(deftest renders-void-elements
  (is (= "<br>" (html/render [:br])))
  (is (= "<img src=\"a.png\">" (html/render [:img {:src "a.png"}])))
  (is (= "<input type=\"text\">" (html/render [:input {:type "text"}]))))

(deftest renders-boolean-attributes
  (is (= "<input disabled>" (html/render [:input {:disabled true}])))
  (is (= "<input>" (html/render [:input {:disabled false}]))))

(deftest renders-nil-as-empty
  (is (= "" (html/render nil))))

(deftest renders-numbers
  (is (= "<span>42</span>" (html/render [:span 42]))))

(deftest renders-sequences
  (is (= "<div><span>a</span><span>b</span></div>"
         (html/render [:div (map (fn [x] [:span x]) ["a" "b"])]))))

(deftest renders-no-attrs
  (is (= "<div><p>hi</p></div>"
         (html/render [:div [:p "hi"]]))))
