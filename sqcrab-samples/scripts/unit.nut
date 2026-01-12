function GetMeter() {
    local hp = this.GetHP();
    print("Player HP is " + hp);
    local mp = this.GetMP();
    print("Player MP is " + mp);
}

function HealHP(n) {
    this.SetHP(this.GetHP() + n);
}

function HealMP(n) {
    this.SetMP(this.GetMP() + n);
}

function MeterChange() {
    this.HealHP(10);
    this.HealMP(5);
}