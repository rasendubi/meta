(ns meta.layout.core
  (:require [clojure.core :refer [lazy-seq]])
  (:refer-clojure :exclude [empty concat]))

(defn empty []
  {:type :empty
   :width 0})

(defn cell [width payload]
  {:type :cell
   :width width
   :payload payload})

(defn indent []
  {:type :indent})

(defn line
  ([] (line (empty)))

  ([alt]
   {:type :line
    :indent (indent)
    :alt alt}))

(defn nest [width doc]
  {:type :nest
   :width width
   :doc doc})

(defn concat [parts]
  {:type :concat
   :parts parts})

(defn group [doc]
  {:type :group
   :doc doc})

(def ^:dynamic ^:private width 80)

(defn- try-fit
  ([pos cmds] (try-fit pos cmds []))
  ([pos cmds result]
   #_(prn "try-fit" pos cmds width)
   (cond
     (> pos width) false

     (empty? cmds) [pos cmds result]

     :else
     (let [[indent-level mode doc] (peek cmds)
           rest (pop cmds)]
       (case (:type doc)
         :empty
         (try-fit pos rest result)

         :cell
         (try-fit (+ pos (:width doc)) rest (conj result doc))

         :concat
         (try-fit pos
                  (apply conj rest
                         (map #(vector indent-level mode %) (reverse (:parts doc))))
                  result)

         :nest
         (try-fit pos
                  (conj rest [(+ indent-level (:width doc)) mode (:doc doc)])
                  result)

         :line
         (if (= mode :break)
           [pos cmds result]
           (try-fit (+ pos (:width (:alt doc))) rest (conj result (:alt doc))))

         :group
         (try-fit pos (conj rest [indent-level :flat (:doc doc)]) result))))))

(defn- do-layout [pos cmds]
  #_(prn "do-layout" pos cmds width)
  (if (empty? cmds)
    '()
    (let [[indent-level mode doc] (peek cmds)
          rest (pop cmds)]
      (case (:type doc)
        :empty
        (do-layout pos rest)

        :cell
        (lazy-seq (cons doc (do-layout pos rest)))

        :concat
        (do-layout pos
                   (apply conj rest
                          (map #(vector indent-level mode %) (reverse (:parts doc)))))

        :line
        (if (= mode :break)
          (lazy-seq (cons (merge doc {:width 0})
                          (cons (merge (:indent doc) {:width indent-level})
                                (do-layout indent-level rest))))
          (lazy-seq (cons (:alt doc)
                          (do-layout (+ pos (:width (:alt doc))) rest))))

        :nest
        (do-layout pos (conj rest [(+ indent-level (:width doc)) mode (:doc doc)]))

        :group
        (if-let [[next-pos next-cmds result] (try-fit pos (conj rest [indent-level :flat (:doc doc)]))]
          (clojure.core/concat result (lazy-seq (do-layout next-pos next-cmds)))
          (do-layout pos (conj rest [indent-level :break (:doc doc)])))))))

(defn layout
  ([doc] (layout doc 80))

  ([doc width]
   #_(prn "layout" doc)
   (binding [meta.layout.core/width width]
     (do-layout 0 [[0 :break doc]]))))
