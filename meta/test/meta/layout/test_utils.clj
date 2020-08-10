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
       (prn actual# expected#)
       (if (= actual# expected#)
         (clojure.test/do-report {:type :pass
                                  :expected expected#
                                  :actual actual#
                                  :message ~msg})
         (clojure.test/do-report {:type :fail
                                  :expected expected#
                                  :actual actual#
                                  :message ~msg})))))

#_(:clj
   (defmethod t/assert-expr 'layout= [msg form]
     (prn "assert-expr" form)
     (let [doc (nth form 1)
           simple-doc (drop 2 form)]
       `(let [actual# (l/layout ~doc)
              expected# (list ~@simple-doc)]
          (prn actual# expected#)
          (if (= actual# expected#)
            (t/do-report {:type :pass
                          :expected expected#
                          :actual actual#
                          :message ~msg})
            (t/do-report {:type :fail
                          :expected expected#
                          :actual actual#
                          :message ~msg}))))))
