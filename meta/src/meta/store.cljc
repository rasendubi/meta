(ns meta.store
  (:require [clojure.set :refer [union difference]])
  (:refer-clojure :exclude [remove]))

(defrecord Store [eav aev ave])

(defn store? [x]
  (instance? Store x))

(defn- add-to-index [store idx x y z]
  (update-in store [idx x y] union #{z}))

(defn- or-nil [x] (when (seq x) x))
(def ^:private difference-or-nil (comp or-nil difference))
(defn- remove-from-index [store idx x y z]
  (update-in store [idx x y] difference-or-nil #{z}))

(defn- apply-to-indices-fn [f]
  (fn [store [e a v]]
    (-> store
        (f :eav e a v)
        (f :aev a e v)
        (f :ave a v e))))

(def ^:private add-datom (apply-to-indices-fn add-to-index))
(def ^:private remove-datom (apply-to-indices-fn remove-from-index))

(defn datoms->store
  "Construct a meta.store from datoms."
  [coll]
  (map->Store (reduce add-datom {} coll)))

(defn- index-lookup-fn [idx]
  (fn
    ([store x]   (get-in store [idx x]))
    ([store x y] (get-in store [idx x y]))))

(def eav
  "Lookup values by entity then attribute"
  (index-lookup-fn :eav))
(def aev
  "Lookup value by attribute then entity"
  (index-lookup-fn :aev))
(def ave
  "Lookup entities by attribute then value"
  (index-lookup-fn :ave))

(defn entities [store]
  (set (keys (:eav store))))

;; updates

(defn remove [store datom]
  (remove-datom store datom))

(defn add [store datom]
  (add-datom store datom))
