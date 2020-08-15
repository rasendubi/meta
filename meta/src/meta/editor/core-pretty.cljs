(ns meta.editor.core-pretty
  (:require [meta.base :as b]
            [meta.core :as c]
            [meta.layout :as l]
            [meta.editor.projectional :as p]))

(def hide-attrs #{c/identifier c/type c/comment})

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (p/editable-cell value)))

(defn- identifier [meta id]
  (editable-text meta id c/identifier))

(defn- annotated-identifier [meta id]
  (l/concat* (identifier meta id)
             (p/round-brackets (p/keyword-cell id))))

(defn pretty-attribute [meta id [attr val]]
  (l/concat*
   p/hardbreak
   (l/nest
    2
    (l/group*
     (annotated-identifier meta attr)
     p/space
     (p/punctuation "=")
     p/line
     (editable-text meta id attr)))))

(defn pretty-entity [meta id]
  (let [avs (filter (comp not hide-attrs first)
                    (b/q '[:find ?a ?v
                           :where
                           (m ?e ?a ?v)]
                         meta
                         [['?e id]]))
        type (c/meta-type meta id)]
    (l/concat*
     (p/punctuation "#")
     p/space
     (editable-text meta id c/comment)
     p/hardbreak

     (l/group*
      (annotated-identifier meta id)
      p/space
      (p/punctuation ":")
      p/space
      (annotated-identifier meta type)
      p/space
      (p/curly-brackets
       (l/concat*
        (l/nest* 2
                 (l/concat (map (partial pretty-attribute meta id) avs)))
        p/break)))
     p/hardbreak)))

(defn pretty-entities [meta ids]
  (l/concat (interpose p/hardbreak (map (partial pretty-entity meta) ids))))
