(ns meta.editor.datoms-view
  (:require [meta.base :as b]
            [meta.core]
            [meta.editor.common :refer [db]]))

(defn annotate [e]
  (let [id (b/q '[:find ?v .
                  :where
                  (m ?e "0" ?v)]
                db
                [['?e e]])]
    [:span [:span.editor-entity id] [:span.editor-dim "("] e [:span.editor-dim ")"]]))

(defn annotate-value [a v]
  (let [value-type (b/q '[:find ?type .
                          :where
                          (m ?a "1" ?type)]
                        db
                        [['?a a]])]
    (condp = value-type
      meta.core/String        [:span.editor-value (str "\"" v "\"")]
      meta.core/Reference     [annotate v]
      meta.core/NaturalNumber [:span.editor-value (str v)]
      meta.core/IntegerNumber [:span.editor-value (str v)]
      )))

(defn annotation [e a v]
  [:div
   ;; [:span.editor-lead e " " a " " v]
   [:span [annotate e] "." [annotate a] " = " [annotate-value a v]]])

(defn datom [x]
  (let [[e a v] (b/q '[:find [?e ?a ?v]
                       :where
                       [?m :e ?e]
                       [?m :a ?a]
                       [?m :v ?v]]
                     db
                     [['?m x]])]
    [annotation e a v]))

(defn datoms-list []
  (let [datoms
        (->> (b/q '[:find ?e
                    :where
                    [?e :e _]]
                  db)
             (map first)
             sort)]
    [:div
     (for [x datoms]
       ^{:key x} [datom x])]))
