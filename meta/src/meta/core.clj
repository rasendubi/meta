(ns meta.core
  (:require [meta.base :as b]))

(def identifier "0")
;; TODO: un-hardcode attribute/value-type
(def attribute-value-type "1")
(def attr-type "5")

(defn meta-type [meta id]
  (if-let [ty (b/value meta id attr-type)]
    ty
    (throw (Exception. (str id ": no type found")))))
