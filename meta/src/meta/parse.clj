(ns meta.parse
  (:require [cheshire.core :as json]
            [meta.base :as base]))

(defn- json-parse-maybe [s]
  (try (json/parse-string s)
       (catch Exception e nil)))

(defn meta-parse
  [filename]
  (filter #(not (nil? %)) (map json-parse-maybe (clojure.string/split-lines (slurp filename)))))

(defn meta-read
  [& filenames]
  (let [data (apply concat (map meta-parse filenames))]
    (base/list->meta data)))

(defmacro meta-read* [& filenames]
  (let [data (apply concat (map meta-parse filenames))]
    `(base/list->meta (quote ~data))))
