(define x (lambda (i) (if (= i 100) i (x (+ i 1)))))

(define fib (lambda (i) (if (< i 2) 1 (+ (fib (- i 1)) (fib (- i 2))))))

(define fact (lambda (x) (if (= x 1) 1 (* x (fact (- x 1))))))

(define fact-iter (lambda (x t) (if (= x 1) t (fact-iter (- x 1) (* t x)))))

(define (factorial x)
  (define (iter x i)
    (if (= x 1)
        i
        (iter (- x 1) (* i x))))
  (iter x 1))
