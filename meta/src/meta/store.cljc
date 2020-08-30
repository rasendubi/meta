(ns meta.store
  (:require [clojure.set :refer [union]]))

(defrecord Store [eav aev ave])

(defn store? [x]
  (instance? Store x))

(defn- update-index [store idx x y z]
  (update-in store [idx x y] union #{z}))

(defn- add-datom [store [e a v]]
  (-> store
      (update-index :eav e a v)
      (update-index :aev a e v)
      (update-index :ave a v e)))

(defn datoms->store
  "Construct a meta.store from datoms."
  [coll]
  (map->Store (reduce add-datom {} coll)))

(defn- index-lookup [idx]
  (fn
    ([store x]   (get-in store [idx x]))
    ([store x y] (get-in store [idx x y]))))

(def eav
  "Lookup values by entity then attribute"
  (index-lookup :eav))
(def aev
  "Lookup value by attribute then entity"
  (index-lookup :aev))
(def ave
  "Lookup entities by attribute then value"
  (index-lookup :ave))

(defn entities [store]
  (set (keys (:eav store))))
