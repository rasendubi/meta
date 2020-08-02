(defcustom meta-annotate-column 47
  "Column to append annotations at.

Setting this to 0 will cause all annotations to be stored on preceding
lines.")

(defvar meta-font-lock-keywords nil "font-lock rules for meta language.")
(setq meta-font-lock-keywords
      '((";.*$" . font-lock-comment-face)
        ("\"\\([^\"\\\\]\\|\\\\.\\)*\"" . font-lock-string-face)))

(defun meta--fill-to-column (col)
  "Adds padding up to COL column."
  (let ((cur (current-column)))
    (if (< cur col)
        (insert (make-string (- col cur) ?\s)))))

(defun meta-annotate-line ()
  "Annotate current line."
  (interactive)
  (save-excursion
    (let ((line (thing-at-point 'line t))
          ;; 2 because we pad extra two spaces with " ; "
          (fill-column (- meta-annotate-column 1)))

      (end-of-line)
      (if (< (current-column) fill-column)
          (progn
            (meta--fill-to-column fill-column)
            (insert " ; "))
        (progn
          (goto-char (line-beginning-position))
          (insert ";; \n")
          (backward-char)))

      (call-process
       "racket"
       nil
       t
       nil

       "meta-racket/meta.rkt" "-f" "core.meta" "-f" "f.meta" "-f" "f-test.meta" "--annotate-line" line)
      (delete-char -1))))

(defvar meta-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c C-a") #'meta-annotate-line)
    map))

;; (general-def 'meta-mode-map "<f8>" #'meta-annotate-line)

(define-derived-mode meta-mode text-mode "meta"
  "major mode for editing meta language code."
  (setq font-lock-defaults '(meta-font-lock-keywords)))

(provide 'meta-mode)
