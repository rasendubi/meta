(ns meta.editor.f
  (:require [meta.core :as c]
            [meta.base :as b]
            [meta.f :as f]
            [meta.layout.core :as l]
            [meta.editor.common :refer [db]]))

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (l/cell (count value)
            {:type :editable-text
             :value value})))

(defn- punctuation [text]
  (l/cell (count text)
          {:type :punctuation
           :value text}))

(defn- keyword-cell [text]
  (l/cell (count text)
          {:type :keyword
           :value text}))

(def line (l/line (punctuation " ")))
(def break (l/line))
(def comma (punctuation ","))

(defmulti f-pretty
  "Pretty-print meta.f into a layout document."
  (fn [meta id] (c/meta-type meta id)))

(defmethod f-pretty f/StringLiteral [meta id]
  (l/concat (list (punctuation "\"")
                  (editable-text meta id f/StringLiteral-value)
                  (punctuation "\""))))

(defmethod f-pretty f/RecordLiteral [meta id]
  (let [field-ids (b/values meta id f/RecordLiteral-field)]
    (l/group (l/concat (list (punctuation "{")
                             (l/nest 2 (l/concat
                                        (interpose comma (map #(f-pretty meta %) field-ids))))
                             line
                             (punctuation "}"))))))

(defmethod f-pretty f/RecordField [meta id]
  (let [key-id   (b/value meta id f/RecordField-key)
        value-id (b/value meta id f/RecordField-value)]
    (l/concat
     (list line
           (l/nest 2 (l/concat
                      (list (l/group (l/concat
                                      (list (punctuation "[")
                                            (f-pretty meta key-id)
                                            (punctuation "]")
                                            (punctuation ":")
                                            line
                                            (f-pretty meta value-id)))))))))))

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
                      (punctuation " ")
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
     (punctuation " ")
     (punctuation "{")
     (l/nest 2 (l/concat (map #(f-pretty meta %) binding-ids)))
     line
     (punctuation "}")
     (punctuation " ")
     (keyword-cell "in")
     (l/group (l/nest* 2 line (f-pretty meta value-id))))))

(defmethod f-pretty f/Binding [meta id]
  (let [identifier-id (b/value meta id f/Binding-identifier)
        value-id      (b/value meta id f/Binding-value)]
    (l/concat*
     line
     (l/group (l/nest* 2
                       (f-pretty meta identifier-id)
                       (punctuation " ")
                       (punctuation "=")
                       line
                       (f-pretty meta value-id)
                       (punctuation ";"))))))

(defn- pretty->string [x]
  (case (:type x)
    :empty ""

    :line "\n"

    :indent (apply str (repeat (:width x) " "))

    :cell
    (:value (:payload x))))

(defn f [id]
  (let [document (f-pretty db id)
        simple   (l/layout document 30)
        string   (apply str (map pretty->string simple))]
    (prn simple)
    string))
