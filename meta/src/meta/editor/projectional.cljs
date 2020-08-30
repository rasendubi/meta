(ns meta.editor.projectional
  (:require [reagent.core :as r]
            [meta.editor.projectional.pretty :as pretty]
            [meta.editor.projectional.view :as view]
            [meta.pathify :as pathify]))

(def projectional view/projectional)

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
