(ns meta.editor.projectional
  (:require [meta.layout :as l]))

(defn surround
  ([left right]
   (fn [doc]
     (surround left right doc)))

  ([left right doc]
   (l/concat (list left doc right))))

;; (def line (l/line (punctuation " ")))
;; (def break (l/line))
;; (def comma (punctuation ","))
;;
;; (def square-brackets (surround "[" "]"))
;; (def round-brackets  (surround "(" ")"))
;; (def curly-brackets  (surround "{" "}"))

(defn cell [class text]
  (l/cell (count text)
          {:class class
           :value text}))
