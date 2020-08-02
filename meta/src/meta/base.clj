(ns meta.base
  (:require [clojure.string :refer [split-lines]]
            [cheshire.core :as json]
            [datascript.core :as d]))

(defn- json-parse-maybe [s]
  (try (json/parse-string s)
       (catch Exception e nil)))

(defn- meta-parse
  [filename]
  (filter #(not (nil? %)) (map json-parse-maybe (clojure.string/split-lines (slurp filename)))))

(defn- ->datascript [[e a v]]
  {:e e :a a :v v})

(defn meta-read
  [& filenames]
  (let [data (apply concat (map meta-parse filenames))]
    (d/db-with (d/empty-db {}) (map ->datascript data))))

(defn q
  ([query db]
   (q query db '[] '[]))

  ([query db bindings]
   (q query db bindings '[]))

  ([query db bindings rules]
   ;; (println (str "query: " query ", bindings: " bindings))
   (let [result
         (apply d/q
                (into (into '[:in $ %] (map first bindings)) query)
                db
                (into '[[(m ?e ?a ?v)
                         [?x :e ?e]
                         [?x :a ?a]
                         [?x :v ?v]]]
                      rules)
                (map second bindings))]
     ;; (println (str "result: " result))
     result)))

(defn values [db e a]
  (map first (q '[:find ?v :where (m ?e ?a ?v)] db [['?e e] ['?a a]])))

(defn value [db e a]
  (q '[:find ?v . :where (m ?e ?a ?v)] db [['?e e] ['?a a]]))
