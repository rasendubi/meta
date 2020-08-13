(ns meta.editor.f
  (:require [reagent.core :as r]
            [meta.layout :as l]
            [meta.pathify :as pathify]
            [meta.editor.f-pretty :refer [f-pretty cell-priority]]
            [meta.editor.common :refer [db]]))

(def f-current-root
  "Id of the root element to display."
  (r/atom nil))

(defn- split-by [pred coll]
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

(def ^:private layout-2d (r/atom []))

(add-watch f-current-root :calculate-layout
           (fn [_key _ref _old root]
             (let [document (pathify/pathify (f-pretty @db root))
                   simple-doc (l/layout document 30)
                   layout (vec (split-by #(= (:type %) :line) simple-doc))]
               (reset! layout-2d layout))))

(def ^:private cursor-position (r/atom {:row 9 :col 5}))
(defn- move-cursor [drow dcol]
  (swap! cursor-position #(merge-with + % {:row drow :col dcol})))

(defn- cursor-to-cell
  "Get information about the cells around the cursor."
  [layout-2d {:keys [row col]}]
  (let [line (get layout-2d row nil)
        cell (reduce (fn [{:keys [pos after] :as s} c]
                       (let [next-pos (- pos (:width c))]
                         (cond
                           (= 0 (:width c))
                           s

                           (and after (= pos 0))
                           (reduced {:after after :before c})

                           (= next-pos 0)
                           {:after c :pos next-pos}

                           (<= next-pos 0)
                           (reduced {:inside c :pos pos})

                           :else
                           {:after c :pos next-pos})))
                     {:pos col}
                     line)]
    cell))

(def ^:private current-cell (r/atom nil))

(defn- find-current-cell
  "Select current cell according to `cursor-to-cell` and `cell-priority`."
  [layout cursor]
  (let [{:keys [inside before after]} (cursor-to-cell layout cursor)]
    (if (and before after)
      (if (< (cell-priority before) (cell-priority after))
        after
        before)
      (or inside before after))))

(add-watch cursor-position :current-cell
           (fn [_key _ref _old cursor-position]
             (reset! current-cell (find-current-cell @layout-2d cursor-position))))
(add-watch layout-2d :current-cell
           (fn [_key _ref _old layout-2d]
             (reset! current-cell (find-current-cell layout-2d @cursor-position))))

(def ^:private line-height 1.28125)
(defn- cursor []
  (prn @current-cell)
  (let [{:keys [row col]} @cursor-position]
    [:div.cursor {:style {:position :absolute
                          :left (str col "ch")
                          :top (str (* row line-height) "em")
                          :height (str line-height "em")
                          :background :black
                          :width "1px"}}]))

(defn- handle-event [e]
  (case (:key e)
    "ArrowLeft"  (move-cursor 0 -1)
    "ArrowDown"  (move-cursor 1 0)
    "ArrowUp"    (move-cursor -1 0)
    "ArrowRight" (move-cursor 0 1)
    nil))

(defn- event->cljs [e]
  {:key       (.-key e)
   :alt       (.-altKey e)
   :ctrl      (.-ctrlKey e)
   :meta      (.-metaKey e)
   :shift     (.-shiftKey e)
   :repeat    (.-repeat e)
   :composing (.-isComposing e)})

(defn- hidden-input []
  [:div #_{:style {:width 0
                   :height 0
                   :overflow :hidden}}
   [:input {:onKeyDown (fn [x] (handle-event (event->cljs x)))
            :autoFocus true}]])

(defn- f-cell [x]
  (let [current-class (when (= x @current-cell) :f-current-cell)]
    (case (:type x)
      :empty
      nil

      :line
      nil

      :indent
      [:span {:class current-class} (apply str (repeat (:width x) " "))]

      :cell
      (let [cell (:payload x)
            value (:value cell)]
        [:span {:class [current-class (:class cell)]} value]))))

(defn f [id]
  [:div {:style {:position :relative}}
   [cursor]
   (for [line @layout-2d]
     [:div (for [cell line]
             [f-cell cell])])
   [hidden-input]])
