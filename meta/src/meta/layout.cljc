(ns meta.layout
  (:require [clojure.core :refer [lazy-seq]])
  (:refer-clojure :exclude [empty concat]))

(defn empty []
  {:type :empty
   :width 0})

(defn cell [width payload]
  {:type :cell
   :width width
   :payload payload})

(defn indent
  ([]
   {:type :indent})
  ([width]
   {:type :indent
    :width width}))

(defn line
  ([] (line (empty)))

  ([alt]
   {:type :line
    :indent (indent)
    :alt alt
    :width 0}))

(defn nest [indent-level doc]
  {:type :nest
   :indent-level indent-level
   :doc doc})

(defn concat [parts]
  {:type :concat
   :parts parts})

(defn group [doc]
  {:type :group
   :doc doc})

(defn concat* [& args]
  (concat args))

(defn nest* [indent-level & args]
  (nest indent-level (concat args)))

(defn group* [& args]
  (group (concat args)))

(defn- try-fit
  ([pos cmds width] (try-fit pos cmds width []))
  ([pos cmds width result]
   #_(prn "try-fit" pos cmds width)
   (cond
     (> pos width) false

     (empty? cmds) [pos cmds result]

     :else
     (let [[indent-level mode doc] (peek cmds)
           rest (pop cmds)]
       (case (:type doc)
         :empty
         (try-fit pos rest width result)

         :cell
         (try-fit (+ pos (:width doc)) rest width (conj result doc))

         :concat
         (try-fit pos
                  (apply conj rest
                         (map #(vector indent-level mode %) (reverse (:parts doc))))
                  width
                  result)

         :nest
         (try-fit pos
                  (conj rest [(+ indent-level (:indent-level doc)) mode (:doc doc)])
                  width
                  result)

         :line
         (if (= mode :break)
           [pos cmds result]
           (if (:alt doc)
             (try-fit (+ pos (:width (:alt doc))) rest width (conj result (:alt doc)))

             ;; :alt = nil means that's a hard-break
             false))

         :group
         (try-fit pos (conj rest [indent-level :flat (:doc doc)]) width result))))))

(defn- do-layout [pos cmds width]
  #_(prn "do-layout" pos cmds width)
  (if (empty? cmds)
    '()
    (let [[indent-level mode doc] (peek cmds)
          rest (pop cmds)]
      (case (:type doc)
        :empty
        (do-layout pos rest width)

        :cell
        (lazy-seq (cons doc (do-layout pos rest width)))

        :concat
        (do-layout pos
                   (apply conj rest
                          (map #(vector indent-level mode %) (reverse (:parts doc))))
                   width)

        :line
        (if (= mode :break)
          (lazy-seq (cons (merge doc {:width 0})
                          (cons (merge (:indent doc) {:width indent-level})
                                (do-layout indent-level rest width))))
          (lazy-seq (cons (:alt doc)
                          (do-layout (+ pos (:width (:alt doc))) rest width))))

        :nest
        (do-layout pos (conj rest [(+ indent-level (:indent-level doc)) mode (:doc doc)]) width)

        :group
        (if-let [[next-pos next-cmds result] (try-fit pos (conj rest [indent-level :flat (:doc doc)]) width)]
          (clojure.core/concat result (lazy-seq (do-layout next-pos next-cmds width)))
          (do-layout pos (conj rest [indent-level :break (:doc doc)]) width))))))

(defn layout
  ([doc] (layout doc 80))

  ([doc width]
   #_(prn "layout" doc)
   (do-layout 0 [[0 :break doc]] width)))
