# Tutorial 10 — Async Timer with Custom Executor

## Output

![Execution result](assets/execution_result.png)


### Penjelasan

Urutan output yang tercetak - `"hey hey"` lebih dulu, baru `"howdy!"`, lalu `"done!"` - terjadi karena cara kerja async di Rust yang bersifat *lazy*. Ketika `spawner.spawn(async { ... })` dipanggil, ia hanya memasukkan future ke dalam antrian channel, bukan langsung menjalankan isi blok async tersebut. Artinya, kode di dalam `async { ... }` belum dieksekusi sama sekali saat baris `spawn` selesai dijalankan. Karena itu, `println!("Fathir's Komputer: hey hey")` yang berada setelah `spawn` langsung dieksekusi secara sinkron di main thread, dan itulah mengapa kalimat `"hey hey"` muncul pertama kali di output.

Setelah `drop(spawner)` dipanggil untuk menutup sisi pengirim channel (menandakan tidak ada task baru lagi), barulah `executor.run()` mulai bekerja. Executor mengambil task dari antrian dan melakukan *polling* terhadap future tersebut. Saat itulah baris pertama di dalam blok async - `println!("Fathir's Komputer: howdy!")` - akhirnya dieksekusi, sehingga `"howdy!"` muncul setelah `"hey hey"`. Eksekusi lalu mencapai `TimerFuture::new(Duration::new(2, 0)).await`, di mana future timer mengembalikan `Poll::Pending` karena timer belum selesai. Task pun ditangguhkan sementara, dan sebuah thread latar belakang tidur selama 2 detik. Setelah 2 detik berlalu, thread tersebut memanggil `waker.wake()` untuk memasukkan kembali task ke antrian. Executor lalu melanjutkan eksekusi dari titik setelah `.await`, dan mencetak `"Fathir's Komputer: done!"` sebagai output terakhir.

Inilah prinsip dasar async Rust: future tidak melakukan apa pun sampai di-*poll*. Semua kode sinkron yang berjalan sebelum `executor.run()` akan selalu selesai lebih dahulu, tidak peduli seberapa awal `spawn` dipanggil.
