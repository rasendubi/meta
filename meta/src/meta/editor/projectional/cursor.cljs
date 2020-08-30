(ns meta.editor.projectional.cursor)

(defn position [row col]
  {:row row :col col})


(defn between [after before]
  {:after after :before before})

(defn inside [cell offset]
  {:inside cell :pos offset})

(defn position-to-cursor
  "Get information about the cells around the cursor."
  [layout {:keys [row col]}]
  (let [line (get layout row nil)
        cell (reduce (fn [{:keys [pos after] :as s} c]
                       (let [next-pos (- pos (:width c))]
                         (cond
                           (= 0 (:width c))
                           s

                           (and after (= pos 0))
                           (reduced (between after c))

                           (= next-pos 0)
                           {:after c :pos next-pos}

                           (<= next-pos 0)
                           (reduced (inside c pos))

                           :else
                           {:after c :pos next-pos})))
                     {:pos col}
                     line)]
    cell))

(defn- cell-priority
  "If cursor is between two cells, cell priority will determine which cell will be selected."
  [x]
  (case (:type x)
    (:empty :line :indent)
    0

    :cell
    (case (:class (:payload x))
      :whitespace 1
      :punctuation 2
      :keyword 3
      :editable 4
      :error 5
      0)))

(defn resolve-priority
  "Select current cell according to `cell-priority`."
  [cursor]
  (let [{:keys [before after]} cursor]
    (cond
      (:inside cursor) cursor

      (and before after)
      (if (< (cell-priority before) (cell-priority after))
        (inside after (:width after))
        (inside before 0))

      before (inside before 0)

      after (inside after (:width after))

      :else nil)))
