use opencv::{core, highgui, imgcodecs, imgproc, objdetect, prelude::*, types, videoio, Result};
use std::{env, thread, time::Duration};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let socket_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_owned());
    let socket = UdpSocket::bind(&socket_addr)
        .await
        .expect("failed to bind to address");

    println!(
        "Listening on: {}",
        socket
            .local_addr()
            .expect("failed to display local address"),
    );

    match socket.connect(&socket_addr).await {
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }

    let window = "Testing Computer Vision Camera Zero";
    highgui::named_window(window, 0)?;
    #[cfg(ocvrs_opencv_branch_32)]
    let (xml, mut cam) = {
        (
            "/usr/share/OpenCV/haarcascades/haarcascade_frontalface_alt.xml".to_owned(),
            videoio::VideoCapture::new_default(0)?, // 0 is the default camera
        )
    };
    #[cfg(not(ocvrs_opencv_branch_32))]
    let (xml, mut cam) = {
        (
            core::find_file("haarcascades/haarcascade_frontalface_alt.xml", true, false)?,
            videoio::VideoCapture::new(0, videoio::CAP_ANY)?, // 0 is the default camera
        )
    };
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut face = objdetect::CascadeClassifier::new(&xml)?;
    let mut image_count: usize = 0;
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.size()?.width == 0 {
            thread::sleep(Duration::from_secs(50));
            continue;
        }
        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
        let mut reduced = Mat::default();
        imgproc::resize(
            &gray,
            &mut reduced,
            core::Size {
                width: 0,
                height: 0,
            },
            0.25f64,
            0.25f64,
            imgproc::INTER_LINEAR,
        )?;
        let mut faces = types::VectorOfRect::new();
        face.detect_multi_scale(
            &reduced,
            &mut faces,
            1.1,
            2,
            objdetect::CASCADE_SCALE_IMAGE,
            core::Size {
                width: 30,
                height: 30,
            },
            core::Size {
                width: 0,
                height: 0,
            },
        )?;
        println!("faces: {}", faces.len());
        for face in faces {
            println!("face {:?}", face);

            let scaled_face = core::Rect {
                x: face.x * 4,
                y: face.y * 4,
                width: face.width * 4,
                height: face.height * 4,
            };
            imgproc::rectangle(
                &mut frame,
                scaled_face,
                core::Scalar::new(0f64, -1f64, -1f64, -1f64),
                1,
                8,
                0,
            )?;

            image_count += 1;
            let screenshot_image_name = format!("opencv_frame_{}.png", image_count);
            let mut screenshot_vector = core::Vector::<i32>::new();
            screenshot_vector.push(imgcodecs::IMWRITE_PAM_FORMAT_GRAYSCALE_ALPHA);
            imgcodecs::imwrite(&screenshot_image_name, &frame, &screenshot_vector)?;

            let dumby_data = vec![0u8; 65];
            match socket.send(&dumby_data).await {
                Ok(_) => {}
                Err(e) => println!("SEND ERROR\n{:#?}", e),
            }
        }

        highgui::imshow(window, &frame)?;
        if highgui::wait_key(10)? > 0 {
            break;
        }
    }
    Ok(())
}
