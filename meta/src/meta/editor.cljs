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
   [editor.entities/entities-list]
   #_[editor.f/f]
   #_[editor.datoms/datoms-list]])

(reset! editor.common/db (meta-read* "../core.meta" "../f.meta" "../f-test.meta"))
(reset! editor.f/f-current-root "1051")
(rd/render [editor] (.getElementById js/document "app"))
