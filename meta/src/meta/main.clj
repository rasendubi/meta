(ns meta.main
  (:require [meta.parse]
            [meta.f]
            [clojure.pprint :as pp]
            [clojure.stacktrace]))

(defn -main []
  (let [meta (meta.parse/meta-read "../core.meta" "../f.meta" "../f-test.meta")]
    (try
      (pp/pprint (meta.f/f-force-deep (meta.f/f-expr meta "1043")))
      (catch Exception e
        (clojure.stacktrace/print-cause-trace e)))))
