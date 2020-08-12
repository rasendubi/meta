(ns meta.editor
  (:require [meta.editor.datoms-view :as editor.datoms]
            [meta.editor.entities-view :as editor.entities]
            [meta.editor.f :as editor.f]
            [meta.editor.common :as editor.common]
            [reagent.dom :as rd])
  (:require-macros [meta.parse :refer [meta-read*]]))

(enable-console-print!)

(defn editor []
  [:div.editor
   #_[editor.entities/entities-list]
   [editor.f/f "1051"]
   #_[editor.datoms/datoms-list]])

(binding [editor.common/db (meta-read* "../core.meta" "../f.meta" "../f-test.meta")]
  (rd/render [editor] (.getElementById js/document "app")))
