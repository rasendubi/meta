(ns meta.editor.projectional
  (:require [reagent.core :as r]
            [meta.editor.projectional.pretty :as pretty]
            [meta.pathify :as pathify]))

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

(defn- split-lines [x]
  (split-by #(= (:type %) :line) x))

(defn doc->layout
  ([doc] (doc->layout doc 80))
  ([doc width]
   (-> doc
       (pathify/pathify)
       (pretty/layout width)
       (split-lines)
       (vec))))

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

(defn- enumerate [coll]
  (map-indexed (fn [id x] [x id]) coll))

(defn- hidden-input []
  [:div {:style {;; :width 0
                 ;; :height 0
                 :overflow :hidden
                 :top 0
                 :left 400
                 :position :absolute}}
   [:input #_{:onKeyDown (fn [x] (handle-event (event->cljs x)))
              :autoFocus true}]])

(defn projectional [layout]
  [:div {:style {:position :relative}}
   [hidden-input]
   (for [[line i] (enumerate layout)]
     ^{:key (if (seq line) (:path (first line)) i)}
     [:div.line (for [cell line]
                  ^{:key (:path cell)}
                  [c cell])])])
