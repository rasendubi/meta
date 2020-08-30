(ns meta.editor.entities-view
  (:require [reagent.core :as r]
            [reagent.ratom :as ratom]
            [meta.base :as b]
            [meta.editor.common :refer [db]]
            [meta.editor.projectional :as p]
            [meta.editor.projectional.cursor :as cursor]
            [meta.editor.core-pretty :refer [pretty-entities]]))

(def ^:private document
  (ratom/reaction
   (let [entities (->> @db
                       (b/entities)
                       (sort-by #(js/parseInt % 10)))]
     (pretty-entities @db entities))))

(def ^:private layout (ratom/reaction (p/doc->layout @document)))

(def cursor-position (r/atom (cursor/position 0 0)))
(defn- move-cursor-position [drow dcol]
  (swap! cursor-position #(merge-with + % (cursor/position drow dcol))))

(def selected-cell
  (ratom/reaction
   (cursor/resolve-priority (cursor/position-to-cursor @layout @cursor-position))))

(ratom/track!
 (fn []
   (prn "Current path:" (:path (:inside @selected-cell)))))

(ratom/track!
 (fn []
   (prn "Current cursor:" @selected-cell)))

#_(ratom/track!
   (fn []
     (let [[doc rest] (meta.pathify/resolve-path @document (:path (:inside @selected-cell)))]
       (if rest
         (print "resolved rest:" rest)
         (print "resolved cell:" doc)))))

(defn- has-modifiers [{:keys [ctrl alt shift meta]}]
  (or ctrl alt meta))

(defn- is-simple-edit [e]
  (and (not (has-modifiers e))
       (= 1 (count (:key e)))))

(defn- edit! [cursor key]
  ;; TODO: update for meta.store
  #_(swap! db
           (fn [db]
             (let [cell (get-in cursor [:inside :payload])
                   value (:value cell)
                   pos (:pos cursor)
                   new-value (str (subs value 0 pos) key (subs value pos))

                   entity (:entity cell)
                   attribute (:attribute cell)
                   did (d/q '[:find ?did .
                              :in $ ?e ?a ?v
                              :where
                              [?did :e ?e]
                              [?did :a ?a]
                              [?did :v ?v]]
                            db entity attribute value)]
               (prn 'updating did entity attribute new-value)
               (d/db-with db [[:db/cas did :v value new-value]])))))

(defn- handle-event [e]
  (prn 'handle-event e)
  (let [{:keys [key]} e]
    (cond
      (= key "ArrowLeft")  (move-cursor-position 0 -1)
      (= key "ArrowDown")  (move-cursor-position 1 0)
      (= key "ArrowUp")    (move-cursor-position -1 0)
      (= key "ArrowRight") (move-cursor-position 0 1)

      (and
       (is-simple-edit e)
       (= :editable (get-in @selected-cell [:inside :payload :class] nil)))
      (edit! @selected-cell key)

      :else nil)))

(defn entities-list []
  [p/projectional {:onKeyDown handle-event
                   :cursor-position cursor-position}
   @layout])
