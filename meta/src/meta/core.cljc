(ns meta.core
  (:refer-clojure :exclude [type comment])
  (:require [meta.base :as b]))

(def meta.core/identifier "0")
;; TODO: un-hardcode attribute/value-type and the rest
(def meta.core/Attribute-value-type "1")
(def meta.core/String "2")
(def meta.core/Reference "3")
(def meta.core/comment "4")
(def meta.core/type "5")
(def meta.core/Type "6")
(def meta.core/Attribute "7")
(def meta.core/ValueType "8")
(def meta.core/NaturalNumber "9")
(def meta.core/Attribute-reference-type "10")
(def meta.core/IntegerNumber "11")

(defn meta-identifier [meta id]
  (b/value meta id meta.core/identifier))

(defn meta-type [meta id]
  (if-let [ty (b/value meta id meta.core/type)]
    ty
    (throw (#?(:clj Exception. :cljs js/Error.) (str id ": no type found")))))
