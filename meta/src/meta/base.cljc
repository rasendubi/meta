(ns meta.base
  (:require [clojure.string :refer [split-lines]]
            [meta.store :as store]))

(defn list->meta [data]
  (store/datoms->store data))

(defn entities [db]
  (store/entities db))

(defn entity [db e]
  (store/eav db e))

(defn values [db e a]
  (store/eav db e a))

(defn value [db e a]
  (first (values db e a)))
