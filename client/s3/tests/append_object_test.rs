use common::runtime::Executor;
use s3::s3_client::S3Client;

#[test]
fn test_append_object_by_path()->Result<(), String> {
    let region = String::from("cn-bj-1");
    let endpoint = String::from("s3.test.com:8080");
    let ak = String::from("hehehehe");
    let sk = String::from("hehehehe");
    let target_bucket = String::from("test-bucket");
    let target_object = String::from("test-object-1");
    let object_path = String::from("/home/test_object");
    let append_position: u64 = 0;
    let exec = Executor::create();

    let s3_client = S3Client::new(&region, &endpoint, &ak, &sk);
    let resp = exec.get_runtime().block_on(s3_client.append_object_by_path(&target_bucket, &target_object, &append_position, &object_path));
    match resp {
        Ok(result) => {
            println!("test_append_object_by_path resp is {:?}", result);
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Failed to append object by path, error {:?}", error));
        }
    }
}

#[test]
fn test_append_object()->Result<(), String> {
    let region = String::from("cn-bj-1");
    let endpoint = String::from("s3.test.com:8080");
    let ak = String::from("hehehehe");
    let sk = String::from("hehehehe");
    let target_bucket = String::from("test-bucket");
    let target_object = String::from("test-object-2");
    let append_position: u64 = 0;
    let data: Vec<u8> = "Hello, World!".into();
    let exec = Executor::create();

    let s3_client = S3Client::new(&region, &endpoint, &ak, &sk);
    let resp = exec.get_runtime().block_on(s3_client.append_object(&target_bucket, &target_object, &append_position, &data));
    match resp {
        Ok(result) => {
            println!("test_append_object resp is {:?}", result);
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Failed to append object, error {:?}", error));
        }
    }
}
