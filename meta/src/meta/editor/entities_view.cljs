(ns meta.editor.entities-view
  (:require [meta.base :as b]
            [meta.core :as core]
            [meta.editor.common :refer [db]]
            [meta.editor.datoms-view :as datoms]
            [meta.editor.projectional :as p]
            [meta.editor.core-pretty :refer [pretty-entities]]))

(defn entities-list []
  (let [entities
        (->> (b/q '[:find ?e
                    :where
                    (m ?e _ _)]
                  @db)
             (map first)
             (sort-by #(js/parseInt % 10)))]
    [p/projectional (p/doc->layout (pretty-entities @db entities))]))
