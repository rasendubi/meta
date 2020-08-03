(ns meta.f
  (:require [meta.base :as b]
            [meta.core :as c]))

;; TODO: remove hard-code and find these by identifiers
(def StringLiteral "100")
(def StringLiteral-value "101")
(def RecordLiteral "102")
(def RecordLiteral-field "103")
(def RecordField "104")
(def RecordField-key "105")
(def RecordField-value "106")
(def FieldAccess "107")
(def FieldAccess-record "108")
(def FieldAccess-field "109")
(def Function "110")
(def Function-parameter "111")
(def Function-body "112")
(def Identifier "113")
(def Identifier-name "114")
(def Apply "115")
(def Apply-function "116")
(def Apply-argument "117")
(def Reference "118")
(def Reference-reference "119")
(def Letrec "120")
(def Letrec-binding "121")
(def Letrec-value "122")
(def Binding "123")
(def Binding-identifier "124")
(def Binding-value "125")


(defn- -env-lookup [env identifier]
  (if (empty? env)
    nil
    (if-let [value (get (first env) identifier nil)]
      value
      (-env-lookup (rest env) identifier))))
(defn- env-lookup [env identifier]
  (if-let [value (-env-lookup env identifier)]
    value
    (throw (#?(:clj Exception. :cljs js/Error.) (str "Unable to lookup " identifier " in " env)))))


(defrecord ^:private FFunction [parameter body env])
(defn- f-function [parameter body env]
  (->FFunction parameter body env))


(defrecord ^:private ThunkState [meta id env])
(defn- thunk-state? [s] (instance? ThunkState s))

(defrecord ^:private Thunk [atom])
(defn- thunk? [x] (instance? Thunk x))
(defn- thunk [meta id env]
  (->Thunk (atom (->ThunkState meta id env))))
(defn- thunk-state! [thunk value]
  (reset! (:atom thunk) value))

(defmulti f-eval
  "Evaluate expression to Weak Head Normal Form."
  (fn [meta id env] (c/meta-type meta id)))

(declare f-force)
(defn- thunk-state-evaluate [state]
  (let [result (f-eval (:meta state) (:id state) (:env state))]
    (if (thunk? result)
      (f-force result)
      result)))

(defn f-force
  "Force evaluation of `thunk` to Weak Head Normal Form."
  [thunk]
  (let [state @(:atom thunk)]
    (if (thunk-state? state)
      (let [result (thunk-state-evaluate state)]
        (thunk-state! thunk result)
        result)
      state)))

(defn f-force-deep
  "Force evaluation of `thunk` to Normal Form."
  [thunk]
  (let [whnf (f-force thunk)]
    (cond
      (instance? FFunction whnf) whnf
      (map? whnf) (into (hash-map) (map #(vector (first %) (f-force-deep (second %))) whnf))
      :else whnf)))


(defn f-expr
  ([meta id] (thunk meta id '())))

(defmethod f-eval StringLiteral [meta id env]
  (b/value meta id StringLiteral-value))

(defn- eval-field [meta id env]
  (let [key-id   (b/value meta id RecordField-key)
        value-id (b/value meta id RecordField-value)]
    (vector (f-force-deep (thunk meta key-id env)) (thunk meta value-id env))))

(defmethod f-eval RecordLiteral [meta id env]
  (let [field-ids (b/values meta id RecordLiteral-field)]
    (into (hash-map) (map #(eval-field meta % env) field-ids))))

(defmethod f-eval FieldAccess [meta id env]
  (let [record-id (b/value meta id FieldAccess-record)
        field-id  (b/value meta id FieldAccess-field)
        record (f-force      (thunk meta record-id env))
        field  (f-force-deep (thunk meta field-id env))]
    (get record field)))

(defmethod f-eval Function [meta id env]
  (let [parameter-id (b/value meta id Function-parameter)
        body-id      (b/value meta id Function-body)]
    (f-function parameter-id body-id env)))

(defmethod f-eval Apply [meta id env]
  (let [function-id (b/value meta id Apply-function)
        argument-id (b/value meta id Apply-argument)
        function (f-force (thunk meta function-id env))
        argument (thunk meta argument-id env)
        scope (hash-map (:parameter function) argument)
        new-env (cons scope (:env function))]
    (thunk meta (:body function) new-env)))

(defmethod f-eval Reference [meta id env]
  (let [identifier (b/value meta id Reference-reference)]
    (env-lookup env identifier)))

(defn- binding-identifier [meta id]
  (b/value meta id Binding-identifier))
(defn- binding-value [meta id]
  (b/value meta id Binding-value))

(defmethod f-eval Letrec [meta id env]
  (let [bindings (b/values meta id Letrec-binding)
        binding-identifiers (map #(binding-identifier meta %) bindings)
        ;; create "fake" scope so that binding-values evaluation can capture an env
        scope (into (hash-map) (map #(vector % (->Thunk (atom nil))) binding-identifiers))
        new-env (cons scope env)
        binding-values (map #(thunk meta (binding-value meta %) new-env) bindings)
        value    (b/value meta id Letrec-value)]
    ;; fix scope
    (doseq [[identifier value] (map vector binding-identifiers binding-values)]
      (thunk-state! (get scope identifier) @(:atom value)))

    (thunk meta value new-env)))
