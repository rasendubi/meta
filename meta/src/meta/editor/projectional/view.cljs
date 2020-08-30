(ns meta.editor.projectional.view)

(def ^:private line-height 1.28125)
(defn- cursor [cursor-position]
  (let [{:keys [row col]} @cursor-position]
    [:div.cursor {:style {:position :absolute
                          :left (str col "ch")
                          :top (str (* row line-height) "em")
                          :height (str line-height "em")
                          :background :black
                          :width "1px"}}]))

(defn- c [x]
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
      [:span {:class (:class cell)} value])))

(defn- event->cljs [e]
  {:key       (.-key e)
   :alt       (.-altKey e)
   :ctrl      (.-ctrlKey e)
   :meta      (.-metaKey e)
   :shift     (.-shiftKey e)
   :repeat    (.-repeat e)
   :composing (.-isComposing e)})

(defn- hidden-input [{:keys [onKeyDown]}]
  [:div {:style {;; :width 0
                 ;; :height 0
                 :overflow :hidden
                 :top 0
                 :left 400
                 :position :absolute}}
   [:input {:onKeyDown (fn [x] (onKeyDown (event->cljs x)))
            :autoFocus true}]])

(defn- enumerate [coll]
  (map-indexed (fn [id x] [x id]) coll))

(defn projectional [{:keys [onKeyDown cursor-position]} layout]
  [:div {:style {:position :relative}}
   [cursor cursor-position]
   [hidden-input {:onKeyDown onKeyDown}]
   (for [[line i] (enumerate layout)]
     ^{:key (if (seq line) (:path (first line)) i)}
     ;; TODO: extract row component, so that only necessary rows are
     ;; re-rendered
     [:div.line (for [cell line]
                  ^{:key (:path cell)}
                  [c cell])])])
