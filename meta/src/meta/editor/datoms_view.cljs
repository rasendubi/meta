(ns meta.editor.datoms-view
  (:require [meta.base :as b]
            [meta.core]
            [meta.editor.common :refer [db]]))

(defn annotate [e]
  (let [id (b/value @db e "0")]
    [:span [:span.editor-entity id] [:span.editor-dim "("] e [:span.editor-dim ")"]]))

(defn annotate-value [a v]
  (let [value-type (b/value @db a "1")]
    (condp = value-type
      meta.core/String        [:span.editor-value (str "\"" v "\"")]
      meta.core/Reference     [annotate v]
      meta.core/NaturalNumber [:span.editor-value (str v)]
      meta.core/IntegerNumber [:span.editor-value (str v)])))

(defn annotation [e a v]
  [:div
   ;; [:span.editor-lead e " " a " " v]
   [:span [annotate e] "." [annotate a] " = " [annotate-value a v]]])

(defn datom [x]
  (let [[e a v] [nil nil nil]
        #_(b/q '[:find [?e ?a ?v]
                 :where
                 [?m :e ?e]
                 [?m :a ?a]
                 [?m :v ?v]]
               @db
               [['?m x]])]
    [annotation e a v]))

(defn datoms-list []
  (let [datoms '()
        #_(->> (b/q '[:find ?e
                      :where
                      [?e :e _]]
                    @db)
               (map first)
               sort)]
    [:div
     (for [x datoms]
       ^{:key x} [datom x])]))
