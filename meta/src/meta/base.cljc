(ns meta.base
  (:require [clojure.string :refer [split-lines]]
            [datascript.core :as d]))

(defn- ->datascript [[e a v]]
  {:e e :a a :v v})

(defn list->meta [data]
  (d/db-with (d/empty-db {}) (map ->datascript data)))

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
