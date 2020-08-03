(ns meta.editor.entities-view
  (:require [meta.base :as b]
            [meta.core :as core]
            [meta.editor.common :refer [db]]
            [meta.editor.datoms-view :as datoms]))

(def hide-attrs #{core/identifier core/type core/comment})

(defn attribute [a v]
  [:span [datoms/annotate a] " = " [datoms/annotate-value a v]])

(defn entity [id]
  (let [avs (filter (comp not hide-attrs first)
                    (b/q '[:find ?a ?v
                           :where
                           (m ?e ?a ?v)]
                         db
                         [['?e id]]))
        comment (b/value db id core/comment)
        type (b/value db id core/type)]
    [:div
     (when comment
       [:div.editor-comment "# " comment])
     [:span [datoms/annotate id] " : " [datoms/annotate type]]
     (when (seq avs)
       [:span
        [:span " {"]
        (for [[a v] avs]
          ^{:key [a v]} [:div "  " [attribute a v]])
        [:span "}"]])
     [:div " "]]))

(defn entities-list []
  (let [entities
        (->> (b/q '[:find ?e
                    :where
                    (m ?e _ _)]
                  db)
             (map first)
             (sort-by #(js/parseInt % 10)))]
    [:div
     (for [x entities]
       ^{:key x} [entity x])]))
