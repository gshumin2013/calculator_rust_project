use gtk::prelude::*;
use gtk::{Application, Button, ComboBoxText, Entry, Label, Grid};
use libadwaita::{ApplicationWindow as AdwApplicationWindow, prelude::*};
use mysql::prelude::*;
use mysql::*;
use std::error::Error;

struct Calculation {
    num1: f64,
    num2: f64,
    operation: String,
    result: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Инициализация приложения GTK
    let app = Application::builder()
        .application_id("com.example.calculator")
        .build();

    app.connect_activate(|app| {
        // Создание главного окна
        let window = AdwApplicationWindow::builder()
            .application(app)
            .title("Rust Calculator")
            .default_width(400)
            .default_height(300)
            .build();

        // Создание элементов интерфейса
        let grid = Grid::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .row_spacing(12)
            .column_spacing(12)
            .build();

        let num1_entry = Entry::new();
        let num2_entry = Entry::new();
        let operation_combo = ComboBoxText::new();
        let result_label = Label::new(None);
        let calculate_btn = Button::with_label("Рассчитать");
        let history_btn = Button::with_label("Показать историю");

        // Заполнение выпадающего списка операций
        operation_combo.append_text("+");
        operation_combo.append_text("-");
        operation_combo.append_text("*");
        operation_combo.append_text("/");
        operation_combo.set_active(Some(0)); // "+" по умолчанию

        // Размещение элементов на сетке
        grid.attach(&Label::new(Some("Число 1:")), 0, 0, 1, 1);
        grid.attach(&num1_entry, 1, 0, 1, 1);
        grid.attach(&Label::new(Some("Число 2:")), 0, 1, 1, 1);
        grid.attach(&num2_entry, 1, 1, 1, 1);
        grid.attach(&Label::new(Some("Операция:")), 0, 2, 1, 1);
        grid.attach(&operation_combo, 1, 2, 1, 1);
        grid.attach(&calculate_btn, 0, 3, 2, 1);
        grid.attach(&Label::new(Some("Результат:")), 0, 4, 1, 1);
        grid.attach(&result_label, 1, 4, 1, 1);
        grid.attach(&history_btn, 0, 5, 2, 1);

        // Функция для выполнения расчета
        calculate_btn.connect_clicked(move |_| {
            // Получаем введенные числа
            let num1 = match num1_entry.text().parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    result_label.set_text("Ошибка в числе 1");
                    return;
                }
            };
            
            let num2 = match num2_entry.text().parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    result_label.set_text("Ошибка в числе 2");
                    return;
                }
            };

            // Получаем выбранную операцию
            let operation = match operation_combo.active_text() {
                Some(op) => op.to_string(),
                None => {
                    result_label.set_text("Выберите операцию");
                    return;
                }
            };

            // Выполняем расчет
            let result = match operation.as_str() {
                "+" => num1 + num2,
                "-" => num1 - num2,
                "*" => num1 * num2,
                "/" => if num2 != 0.0 { num1 / num2 } else { f64::NAN },
                _ => {
                    result_label.set_text("Неизвестная операция");
                    return;
                }
            };

            // Отображаем результат
            if result.is_nan() {
                result_label.set_text("Деление на ноль!");
            } else {
                result_label.set_text(&format!("{:.2}", result));
            }

            // Сохраняем в базу данных
            if let Err(e) = save_calculation(num1, num2, &operation, result) {
                eprintln!("Ошибка сохранения: {}", e);
            }
        });

        // Показать историю (выводим в консоль)
        history_btn.connect_clicked(move |_| {
            if let Err(e) = show_history() {
                eprintln!("Ошибка получения истории: {}", e);
            }
        });

        window.set_content(Some(&grid));
        window.present();
    });

    app.run();

    Ok(())
}

fn save_calculation(num1: f64, num2: f64, operation: &str, result: f64) -> Result<(), Box<dyn Error>> {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("mariadb"))  
        .user(Some("calculator_user"))
        .pass(Some("userpassword"))
        .db_name(Some("calculator_db"))
        .tcp_port(3306);  
    
    let pool = Pool::new(opts)?;
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        "INSERT INTO calculations (num1, num2, operation, result) VALUES (?, ?, ?, ?)",
        (num1, num2, operation, result),
    )?;

    Ok(())
}

fn show_history() -> Result<(), Box<dyn Error>> {
    let opts = OptsBuilder::new()
    .ip_or_hostname(Some("mariadb"))  
    .user(Some("calculator_user"))
    .pass(Some("userpassword"))
    .db_name(Some("calculator_db"))
    .tcp_port(3306);
    
    let pool = Pool::new(opts)?;
    let mut conn = pool.get_conn()?;

    let calculations: Vec<(f64, f64, String, f64)> = conn.query_map(
        "SELECT num1, num2, operation, result FROM calculations ORDER BY created_at DESC LIMIT 10",
        |(num1, num2, operation, result)| (num1, num2, operation, result),
    )?;

    println!("Последние 10 вычислений:");
    for (i, (num1, num2, op, res)) in calculations.iter().enumerate() {
        println!("{}: {} {} {} = {}", i+1, num1, op, num2, res);
    }

    Ok(())
}

fn get_db_connection() -> Result<PooledConn, Box<dyn Error>> {
    let db_host = "mariadb"; 
    let db_port = 3306;      
    
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(db_host))
        .tcp_port(db_port)
        .user(Some("calculator_user"))
        .pass(Some("userpassword"))
        .db_name(Some("calculator_db"));

    let pool = Pool::new(opts)?;
    pool.get_conn().map_err(|e| e.into())
}