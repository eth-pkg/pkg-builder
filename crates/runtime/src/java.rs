use config::language::JavaConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct JavaRuntime(pub JavaConfig);

impl Runtime for JavaRuntime {
    fn install_commands(&self) -> Vec<String> {
        let mut s = ScriptBuilder::new();
        s.cmd("apt install -y wget");
        s.cmd(format!(
            "mkdir -p /opt/lib/jvm/jdk-{}-oracle && mkdir -p /usr/lib/jvm",
            self.0.jdk_version
        ));
        s.cmd(format!(
            "cd /tmp && wget -q --output-document jdk.tar.gz {}",
            self.0.jdk_binary_url
        ));
        s.cmd(format!(
            "cd /tmp && echo \"{} jdk.tar.gz\" >>hash_file.txt && cat hash_file.txt",
            self.0.jdk_binary_checksum
        ));
        s.cmd("cd /tmp && sha256sum -c hash_file.txt");
        s.cmd(format!(
            "cd /tmp && tar -zxf jdk.tar.gz -C /opt/lib/jvm/jdk-{}-oracle --strip-components=1",
            self.0.jdk_version
        ));
        s.cmd(format!(
            "ln -s /opt/lib/jvm/jdk-{}-oracle/bin/java /usr/bin/java",
            self.0.jdk_version
        ));
        s.cmd(format!(
            "ln -s /opt/lib/jvm/jdk-{}-oracle/bin/javac /usr/bin/javac",
            self.0.jdk_version
        ));
        s.cmd("java -version");
        s.cmd("apt remove -y wget");

        // Gradle support
        if let Some(ref gradle) = self.0.gradle {
            s.cmd("apt install -y wget unzip");
            s.cmd(format!(
                "mkdir -p /opt/lib/gradle-{}",
                gradle.gradle_version
            ));
            s.cmd(format!(
                "cd /tmp && wget -q --output-document gradle.tar.gz {}",
                gradle.gradle_binary_url
            ));
            s.cmd(format!(
                "cd /tmp && echo \"{} gradle.tar.gz\" > hash_file.txt && cat hash_file.txt",
                gradle.gradle_binary_checksum
            ));
            s.cmd("cd /tmp && sha256sum -c hash_file.txt");
            s.cmd(format!(
                "cd /tmp && unzip gradle.tar.gz && mv gradle-{} /opt/lib",
                gradle.gradle_version
            ));
            s.cmd(format!(
                "ln -s /opt/lib/gradle-{}/bin/gradle /usr/bin/gradle",
                gradle.gradle_version
            ));
            s.cmd("gradle -version");
            s.cmd("apt remove -y wget");
        }

        s.build()
    }
}
