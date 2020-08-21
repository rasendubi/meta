(ns meta.editor.entities-view
  (:require [reagent.core :as r]
            [meta.base :as b]
            [meta.editor.common :refer [db]]
            [meta.editor.projectional :as p]
            [meta.editor.core-pretty :refer [pretty-entities]]))

(defn- get-core-document []
  (let [entities
        (->> (b/q '[:find ?e
                    :where
                    (m ?e _ _)]
                  @db)
             (map first)
             (sort-by #(js/parseInt % 10)))]
    (pretty-entities @db entities)))
(def ^:private document (r/track get-core-document))

(defn- get-core-layout []
  (p/doc->layout @document))
(def ^:private layout-2d (r/track get-core-layout))


(defn entities-list []
  [p/projectional @layout-2d])
