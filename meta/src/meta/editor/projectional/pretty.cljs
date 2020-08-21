(ns meta.editor.projectional.pretty
  (:require [meta.layout :as l])
  (:refer-clojure :exclude [empty concat]))

(def empty l/empty)
(def nest l/nest)
(def nest* l/nest*)
(def concat l/concat)
(def concat* l/concat*)
(def group l/group)
(def group* l/group*)

(def layout l/layout)

(defn cell
  ([text]
   (cell text {}))

  ([text extra]
   (l/cell (count text)
           (merge {:value text} extra))))

(defn- cell-with [meta]
  (fn
    ([text]       (cell text meta))
    ([text extra] (cell text (merge meta extra)))))

(def whitespace (cell-with {:class :whitespace}))
(def punctuation (cell-with {:class :punctuation}))
(def keyword-cell (cell-with {:class :keyword}))
(def error-cell (cell-with {:class :error}))
(def editable-cell (cell-with {:class :editable}))

(defn editable-string
  [entity-id attr-id value]
  (cell value {:class :editable
               :entity entity-id
               :attribute attr-id}))

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
