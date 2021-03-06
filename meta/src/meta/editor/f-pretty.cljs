(ns meta.editor.f-pretty
  (:require [meta.base :as b]
            [meta.core :as c]
            [meta.f :as f]
            [meta.editor.projectional.pretty
             :refer [whitespace punctuation keyword-cell error-cell editable-cell space line break comma]
             :as p]))

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (editable-cell value)))

(defn cell-priority
  "If cursor is between two cells, cell priority will determine which cell will be selected."
  [x]
  (case (:type x)
    (:empty :line :indent)
    0

    :cell
    (case (:class (:payload x))
      :whitespace 1
      :punctuation 2
      :keyword 3
      :editable 4
      :error 5)))

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
  (p/concat* (punctuation "\"")
             (editable-text meta id f/StringLiteral-value)
             (punctuation "\"")))

(defmethod f-pretty f/RecordLiteral [meta id]
  (let [field-ids (b/values meta id f/RecordLiteral-field)]
    (p/group* (punctuation "{")
              (p/nest 2 (p/concat
                         (interpose comma (map #(f-pretty meta %) field-ids))))
              line
              (punctuation "}"))))

(defmethod f-pretty f/RecordField [meta id]
  (let [key-id   (b/value meta id f/RecordField-key)
        value-id (b/value meta id f/RecordField-value)]
    (p/concat*
     line
     (p/nest* 2
              (p/group*
               (punctuation "[")
               (f-pretty meta key-id)
               (punctuation "]")
               (punctuation ":")
               line
               (f-pretty meta value-id))))))

(defmethod f-pretty f/FieldAccess [meta id]
  (let [record-id (b/value meta id f/FieldAccess-record)
        field-id  (b/value meta id f/FieldAccess-field)]
    (p/group*
     (f-pretty meta record-id)
     (p/nest* 2
              break
              (punctuation ".")
              (punctuation "[")
              (f-pretty meta field-id)
              (punctuation "]")))))

(defmethod f-pretty f/Function [meta id]
  (let [parameter-id (b/value meta id f/Function-parameter)
        body-id      (b/value meta id f/Function-body)]
    (p/group (p/nest* 2
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
    (p/group*
     (punctuation "(")
     (f-pretty meta function-id)
     (punctuation ")")
     (p/nest* 2
              line
              (punctuation "(")
              (f-pretty meta argument-id)
              (punctuation ")")))))

(defmethod f-pretty f/Reference [meta id]
  (f-pretty meta (b/value meta id f/Reference-reference)))

(defmethod f-pretty f/Letrec [meta id]
  (let [binding-ids (b/values meta id f/Letrec-binding)
        value-id    (b/value  meta id f/Letrec-value)]
    (p/group*
     (keyword-cell "let")
     space
     (punctuation "{")
     (p/nest 2 (p/concat (map #(f-pretty meta %) binding-ids)))
     line
     (punctuation "}")
     space
     (keyword-cell "in")
     (p/group (p/nest* 2 line (f-pretty meta value-id))))))

(defmethod f-pretty f/Binding [meta id]
  (let [identifier-id (b/value meta id f/Binding-identifier)
        value-id      (b/value meta id f/Binding-value)]
    (p/concat*
     line
     (p/group (p/nest* 2
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
