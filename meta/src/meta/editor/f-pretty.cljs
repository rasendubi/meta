(ns meta.editor.f-pretty
  (:require [meta.base :as b]
            [meta.core :as c]
            [meta.layout :as l]
            [meta.f :as f]
            [meta.editor.projectional :as p]))

(def ^:private whitespace (partial p/cell :f-whitespace))
(def ^:private punctuation (partial p/cell :f-punctuation))
(def ^:private keyword-cell (partial p/cell :f-keyword))
(def ^:private error-cell (partial p/cell :f-error))

(def ^:private space (whitespace " "))
(def ^:private line (l/line space))
(def ^:private break (l/line))
(def ^:private comma (punctuation ","))

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (p/cell :f-editable-text value)))

(defn cell-priority
  "If cursor is between two cells, cell priority will determine which cell will be selected."
  [x]
  (case (:type x)
    (:empty :line :indent)
    0

    :cell
    (case (:class (:payload x))
      :f-whitespace 1
      :f-punctuation 2
      :f-keyword 3
      :f-editable-text 4
      :f-error 5)))

(defmulti f-pretty
  "Pretty-print meta.f into a layout document."
  (fn [meta id]
    (try
      (c/meta-type meta id)
      (catch :default e
        :error))))

(defmethod f-pretty :error [meta id]
  (error-cell (str "(" id ")")))

(defmethod f-pretty f/StringLiteral [meta id]
  (l/concat* (punctuation "\"")
             (editable-text meta id f/StringLiteral-value)
             (punctuation "\"")))

(defmethod f-pretty f/RecordLiteral [meta id]
  (let [field-ids (b/values meta id f/RecordLiteral-field)]
    (l/group* (punctuation "{")
              (l/nest 2 (l/concat
                         (interpose comma (map #(f-pretty meta %) field-ids))))
              line
              (punctuation "}"))))

(defmethod f-pretty f/RecordField [meta id]
  (let [key-id   (b/value meta id f/RecordField-key)
        value-id (b/value meta id f/RecordField-value)]
    (l/concat*
     line
     (l/nest* 2
              (l/group*
               (punctuation "[")
               (f-pretty meta key-id)
               (punctuation "]")
               (punctuation ":")
               line
               (f-pretty meta value-id))))))

(defmethod f-pretty f/FieldAccess [meta id]
  (let [record-id (b/value meta id f/FieldAccess-record)
        field-id  (b/value meta id f/FieldAccess-field)]
    (l/group*
     (f-pretty meta record-id)
     (l/nest* 2
              break
              (punctuation ".")
              (punctuation "[")
              (f-pretty meta field-id)
              (punctuation "]")))))

(defmethod f-pretty f/Function [meta id]
  (let [parameter-id (b/value meta id f/Function-parameter)
        body-id      (b/value meta id f/Function-body)]
    (l/group (l/nest* 2
                      (punctuation "\\")
                      (f-pretty meta parameter-id)
                      space
                      (punctuation "->")
                      line
                      (f-pretty meta body-id)))))

(defmethod f-pretty f/Identifier [meta id]
  (editable-text meta id f/Identifier-name))

(defmethod f-pretty f/Apply [meta id]
  (let [function-id (b/value meta id f/Apply-function)
        argument-id (b/value meta id f/Apply-argument)]
    (l/group*
     (punctuation "(")
     (f-pretty meta function-id)
     (punctuation ")")
     (l/nest* 2
              line
              (punctuation "(")
              (f-pretty meta argument-id)
              (punctuation ")")))))

(defmethod f-pretty f/Reference [meta id]
  (f-pretty meta (b/value meta id f/Reference-reference)))

(defmethod f-pretty f/Letrec [meta id]
  (let [binding-ids (b/values meta id f/Letrec-binding)
        value-id    (b/value  meta id f/Letrec-value)]
    (l/group*
     (keyword-cell "let")
     space
     (punctuation "{")
     (l/nest 2 (l/concat (map #(f-pretty meta %) binding-ids)))
     line
     (punctuation "}")
     space
     (keyword-cell "in")
     (l/group (l/nest* 2 line (f-pretty meta value-id))))))

(defmethod f-pretty f/Binding [meta id]
  (let [identifier-id (b/value meta id f/Binding-identifier)
        value-id      (b/value meta id f/Binding-value)]
    (l/concat*
     line
     (l/group (l/nest* 2
                       (f-pretty meta identifier-id)
                       space
                       (punctuation "=")
                       line
                       (f-pretty meta value-id)
                       (punctuation ";"))))))

(defn- cell->string [x]
  (case (:type x)
    :empty ""

    :line "\n"

    :indent (apply str (repeat (:width x) " "))

    :cell
    (:value (:payload x))))

(defn pretty->string [xs]
  (->> xs
       (map cell->string)
       (apply str)))
