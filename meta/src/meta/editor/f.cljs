(ns meta.editor.f
  (:require [reagent.core :as r]
            [meta.core :as c]
            [meta.base :as b]
            [meta.f :as f]
            [meta.layout :as l]
            [meta.pathify :as pathify]
            [meta.editor.projectional :as p]
            [meta.editor.common :refer [db]]))

(def punctuation (partial p/cell :punctuation))
(def keyword-cell (partial p/cell :keyword))
(def error-cell (partial p/cell :error))

(def line (l/line (punctuation " ")))
(def break (l/line))
(def comma (punctuation ","))

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (p/cell :editable-text value)))

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

(defn- cell->string [x]
  (case (:type x)
    :empty ""

    :line "\n"

    :indent (apply str (repeat (:width x) " "))

    :cell
    (:value (:payload x))))

(defn- pretty->string [xs]
  (->> xs
       (map cell->string)
       (apply str)))

(defn- f-cell [x]
  (case (:type x)
    :empty
    nil

    :line
    nil

    :indent
    [:span (apply str (repeat (:width x) " "))]

    :cell
    (let [cell (:payload x)
          value (:value cell)]
      (case (:class cell)
        :punctuation
        [:span.f-punctuation value]

        :keyword
        [:span.f-keyword value]

        :editable-text
        [:span.f-editable-text value]

        :error
        [:span.f-error value]

        :else
        [:span value]))))

(defn split-by [pred coll]
  (lazy-seq
   (when-let [s (seq coll)]
     (let [!pred (complement pred)
           [xs ys] (split-with !pred s)]
       (if (seq xs)
         (cons xs (split-by pred ys))
         (let [skip (take-while pred s)
               others (drop-while pred s)
               [xs ys] (split-with !pred others)]
           (cons (concat skip xs)
                 (split-by pred ys))))))))

(def cursor-position (r/atom {:row 9 :col 5}))

(defn move-cursor [drow dcol]
  (swap! cursor-position #(merge-with + % {:row drow :col dcol})))

(def line-height 1.28125)
(defn cursor []
  (let [{:keys [row col]} @cursor-position]
    [:div.cursor {:style {:position :absolute
                          :left (str col "ch")
                          :top (str (* row line-height) "em")
                          :height (str line-height "em")
                          :background :black
                          :width "1px"}}]))

(defn handle-event [e]
  (case (:key e)
    "ArrowLeft"  (move-cursor 0 -1)
    "ArrowDown"  (move-cursor 1 0)
    "ArrowUp"    (move-cursor -1 0)
    "ArrowRight" (move-cursor 0 1)
    nil))

(defn event->cljs [e]
  {:key       (.-key e)
   :alt       (.-altKey e)
   :ctrl      (.-ctrlKey e)
   :meta      (.-metaKey e)
   :shift     (.-shiftKey e)
   :repeat    (.-repeat e)
   :composing (.-isComposing e)})

(defn hidden-input []
  [:div {:style {:width 0
                 :height 0
                 :overflow :hidden}}
   [:input {:onKeyDown (fn [x] (handle-event (event->cljs x)))
            :autoFocus true}]])

(defn- f-editor [document]
  (let [simple (l/layout document 30)]
    [:div {:style {:position :relative}}
     [hidden-input]
     [cursor]
     (for [line (split-by #(= (:type %) :line) simple)]
       [:div (for [cell line]
               [f-cell cell])])]))

(defn f [id]
  (let [document (pathify/pathify (f-pretty db id))]
    [f-editor document]))
