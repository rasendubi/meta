(ns meta.editor
  (:require [meta.f :as f])
  (:require-macros [meta.parse :refer [meta-read*]]))

(enable-console-print!)

(def db (meta-read* "../core.meta" "../f.meta" "../f-test.meta"))

(prn (f/f-force-deep (f/f-expr db "1043")))

(prn "Hi, there")
