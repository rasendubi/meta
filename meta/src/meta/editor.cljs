(ns meta.editor
  (:require [meta.base :as b]
            [meta.f :as f]
            [meta.editor.datoms-view :as editor.datoms]
            [meta.editor.common :as editor.common]
            [reagent.core :as r]
            [reagent.dom :as rd])
  (:require-macros [meta.parse :refer [meta-read*]]))

(enable-console-print!)


(binding [editor.common/db (meta-read* "../core.meta" "../f.meta" "../f-test.meta")]
  (rd/render [editor.datoms/datoms-list] (.getElementById js/document "app")))
