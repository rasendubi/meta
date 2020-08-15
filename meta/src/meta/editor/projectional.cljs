(ns meta.editor.projectional
  (:require [meta.layout :as l]
            [meta.pathify :as pathify]))

(defn cell [class text]
  (l/cell (count text)
          {:class class
           :value text}))

(def whitespace (partial cell :whitespace))
(def punctuation (partial cell :punctuation))
(def keyword-cell (partial cell :keyword))
(def error-cell (partial cell :error))
(def editable-cell (partial cell :editable))

(def space (whitespace " "))
(def line (l/line space))
(def break (l/line))
(def hardbreak (l/line nil))
(def comma (punctuation ","))

(defn surround
  ([left right]
   (fn [doc]
     (surround left right doc)))

  ([left right doc]
   (l/concat* left doc right)))

(def square-brackets (surround (punctuation "[") (punctuation "]")))
(def round-brackets  (surround (punctuation "(") (punctuation ")")))
(def curly-brackets  (surround (punctuation "{") (punctuation "}")))


(defn- split-by [pred coll]
  (lazy-seq
   (when-let [s (seq coll)]
     (let [!pred (complement pred)
           [xs ys] (split-with !pred s)]
       (if (seq xs)
         (cons xs (split-by pred ys))
         (let [skip (take-while pred s)
               others (drop-while pred s)
               [xs ys] (split-with !pred others)]
           (cons (concat skip xs)
                 (split-by pred ys))))))))

(defn doc->layout
  ([doc] (doc->layout doc 80))
  ([doc width]
   (as-> doc x
     (pathify/pathify x)
     (l/layout x width)
     (vec (split-by #(= (:type %) :line) x)))))

(defn- c [x]
  (case (:type x)
    :empty
    nil

    :line
    nil

    :indent
    [:span (apply str (repeat (:width x) " "))]

    :cell
    (let [cell (:payload x)
          value (:value cell)]
      [:span {:class (:class cell)} value])))

(defn projectional [layout]
  [:div {:style {:position :relative}}
   (for [line layout]
     [:div.line (for [cell line]
                  [c cell])])])
