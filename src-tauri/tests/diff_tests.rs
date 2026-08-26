use copiar_lib::models::DiffStatus;
use copiar_lib::services::diff::compare_directories;
use std::fs::{self, File};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_diff_engine_categorization() {
    let src_dir = tempdir().unwrap();
    let tgt_dir = tempdir().unwrap();

    // 1. Arquivo Novo Apenas na Origem
    let only_src = src_dir.path().join("new_in_src.txt");
    let mut f1 = File::create(&only_src).unwrap();
    f1.write_all(b"Hello from source only").unwrap();

    // 2. Arquivo Apenas no Destino
    let only_tgt = tgt_dir.path().join("orphaned_target.txt");
    let mut f2 = File::create(&only_tgt).unwrap();
    f2.write_all(b"Target only").unwrap();

    // 3. Arquivo Idêntico (mesmo tamanho e data)
    let equal_src = src_dir.path().join("equal.txt");
    let equal_tgt = tgt_dir.path().join("equal.txt");
    fs::write(&equal_src, b"Exact same content 12345").unwrap();
    fs::write(&equal_tgt, b"Exact same content 12345").unwrap();
    // Alinhar mtime
    let src_meta = fs::metadata(&equal_src).unwrap();
    let ft = filetime::FileTime::from_system_time(src_meta.modified().unwrap());
    filetime::set_file_mtime(&equal_tgt, ft).unwrap();

    // 4. Arquivo Mais Recente na Origem
    let newer_src = src_dir.path().join("newer.txt");
    let newer_tgt = tgt_dir.path().join("newer.txt");
    fs::write(&newer_tgt, b"Old version 1").unwrap();
    sleep(Duration::from_millis(2000));
    fs::write(&newer_src, b"New version 2").unwrap();

    // 5. Arquivo Mais Pesado na Origem
    let heavy_src = src_dir.path().join("heavy.txt");
    let heavy_tgt = tgt_dir.path().join("heavy.txt");
    fs::write(&heavy_tgt, b"small").unwrap();
    fs::write(&heavy_src, b"large content heavier in source").unwrap();
    // forçar mesma data para testar o status HeavyInSource
    let ft_heavy = filetime::FileTime::from_system_time(fs::metadata(&heavy_tgt).unwrap().modified().unwrap());
    filetime::set_file_mtime(&heavy_src, ft_heavy).unwrap();

    // Executar comparador
    let result = compare_directories(src_dir.path(), tgt_dir.path(), |_| {}).unwrap();

    assert_eq!(result.summary.total_items, 5);
    assert_eq!(result.summary.only_source_count, 1);
    assert_eq!(result.summary.only_target_count, 1);
    assert_eq!(result.summary.equal_count, 1);
    assert_eq!(result.summary.newer_count, 1);
    assert_eq!(result.summary.heavy_count, 1);

    // Verificar itens individuais
    let new_src_item = result.files.iter().find(|f| f.rel_path == "new_in_src.txt").unwrap();
    assert_eq!(new_src_item.status, DiffStatus::OnlyInSource);
    assert!(new_src_item.selected);

    let equal_item = result.files.iter().find(|f| f.rel_path == "equal.txt").unwrap();
    assert_eq!(equal_item.status, DiffStatus::Equal);
    assert!(!equal_item.selected);

    let newer_item = result.files.iter().find(|f| f.rel_path == "newer.txt").unwrap();
    assert_eq!(newer_item.status, DiffStatus::NewerInSource);
    assert!(newer_item.selected);
}
