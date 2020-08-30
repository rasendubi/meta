(ns meta.editor.core-pretty
  (:require [meta.base :as b]
            [meta.core :as c]
            [meta.editor.projectional.pretty :as p]))

(def hide-attrs #{c/identifier c/type c/comment})

(defn- editable-text
  [meta id attr]
  (let [value (b/value meta id attr)]
    (p/editable-string id attr value)))

(defn- identifier [meta id]
  (editable-text meta id c/identifier))

(defn- annotated-identifier [meta id]
  (p/concat* (identifier meta id)
             (p/round-brackets (p/keyword-cell id))))

(defn pretty-attribute [meta id [attr val]]
  (p/concat*
   p/hardbreak
   (p/nest
    2
    (p/group*
     (annotated-identifier meta attr)
     p/space
     (p/punctuation "=")
     p/line
     (editable-text meta id attr)))))

(defn pretty-entity [meta id]
  (let [avs (apply dissoc (b/entity meta id) hide-attrs)
        type (c/meta-type meta id)]
    (p/concat*
     (p/punctuation "#")
     p/space
     (editable-text meta id c/comment)
     p/hardbreak

     (p/group*
      (annotated-identifier meta id)
      p/space
      (p/punctuation ":")
      p/space
      (annotated-identifier meta type)
      p/space
      (p/curly-brackets
       (p/concat*
        (p/nest* 2
                 (p/concat (map (partial pretty-attribute meta id) avs)))
        p/break)))
     p/hardbreak)))

(defn pretty-entities [meta ids]
  (p/concat (interpose p/hardbreak (map (partial pretty-entity meta) ids))))
