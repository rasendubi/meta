(ns meta.pathify)

(defn pathify
  ([doc] (pathify [] doc))

  ([path doc]
   (and
    doc
    (merge
     doc
     {:path path}
     (case (:type doc)
       (:empty :cell :indent)
       nil

       :nest
       {:doc (pathify (conj path :nest) (:doc doc))}

       :group
       {:doc (pathify (conj path :group) (:doc doc))}

       :line
       {:alt (pathify (conj path :alt) (:alt doc))
        :indent (pathify (conj path :indent) (:indent doc))}

       :concat
       {:parts (map-indexed (fn [i x]
                              (pathify (conj path (:key (meta x) i)) x))
                            (:parts doc))})))))

(defn resolve-path-segment
  "Resolve one segment of the path in the `doc`.

  Return next document on success, `nil` if segment can't be found."
  [doc segment]
  (case (:type doc)
    :nest
    (when (= segment :nest)
      (:doc doc))

    :group
    (when (= segment :group)
      (:doc doc))

    :concat
    (if (number? segment)
      (nth (:parts doc) segment nil)
      (first (filter #(= segment (:key (meta %))) (:parts doc))))

    :line
    (case segment
      :alt (:alt doc)
      :indent (:indent doc)
      nil)

    (:indent :empty :cell)
    nil))

(defn resolve-path
  "Follow the `path` starting from the `doc`.

  Returns two-vector, where first element is the last matching
  element, and second elemen is the rest of the path to follow. If the
  path has resolved completely, the second element is `nil`."

  ;; TODO: consider returning already traversed path. It might be
  ;; useful to traverse the same path in a similar document.
  [doc path]
  (if (seq path)
    (let [[cur & rest] path]
      (if-let [next (resolve-path-segment doc cur)]
        (resolve-path next rest)
        [doc path]))
    [doc nil]))
