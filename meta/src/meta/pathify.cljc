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
