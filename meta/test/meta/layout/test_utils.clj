(ns meta.layout.test-utils
  (:require [cljs.test]
            [clojure.test]
            [meta.layout.core :as l]))

(defmethod cljs.test/assert-expr 'layout= [env msg form]
  (let [doc (nth form 1)
        simple-doc (drop 2 form)]
    `(let [actual# (l/layout ~doc 20)
           expected# (list ~@simple-doc)]
       (if (= actual# expected#)
         (cljs.test/do-report {:type :pass
                               :expected expected#
                               :actual actual#
                               :message ~msg})
         (cljs.test/do-report {:type :fail
                               :expected expected#
                               :actual actual#
                               :message ~msg})))))

(defmethod clojure.test/assert-expr 'layout= [msg form]
  (let [doc (nth form 1)
        simple-doc (drop 2 form)]
    `(let [actual# (l/layout ~doc 20)
           expected# (list ~@simple-doc)]
       (if (= actual# expected#)
         (clojure.test/do-report {:type :pass
                                  :expected expected#
                                  :actual actual#
                                  :message ~msg})
         (clojure.test/do-report {:type :fail
                                  :expected expected#
                                  :actual actual#
                                  :message ~msg})))))
